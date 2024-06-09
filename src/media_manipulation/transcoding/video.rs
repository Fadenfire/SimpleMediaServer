use std::ops::Range;

use anyhow::{anyhow, Context};
use ffmpeg_next as ffmpeg;
use ffmpeg_next::{codec, Dictionary, filter, format, frame, Packet, picture, Rational, Rescale};
use ffmpeg_next::format::Pixel;

use crate::media_manipulation::backends::VideoBackend;

const DEFAULT_PIXEL_FORMAT: Pixel = Pixel::YUV420P;

pub struct VideoTranscoder {
	decoder: codec::decoder::Video,
	encoder: codec::encoder::Video,
	filter: filter::graph::Graph,
	
	out_stream_index: usize,
	rate_time_base: Rational,
	first_frame: bool,
	last_dts: i64,
}

pub struct VideoTranscoderParams<'a, B: VideoBackend> {
	pub in_stream: &'a ffmpeg::Stream<'a>,
	pub muxer: &'a mut format::context::Output,
	pub backend: &'a mut B,
	pub output_codec: codec::Id,
	pub bit_rate: usize,
	pub encoder_options: Dictionary<'a>,
}

impl VideoTranscoder {
	pub fn new(params: VideoTranscoderParams<impl VideoBackend>) -> anyhow::Result<Self> {
		let has_global_header = params.muxer.format().flags().contains(format::flag::Flags::GLOBAL_HEADER);
		
		let decoder = params.backend.create_decoder(params.in_stream.parameters(), params.in_stream.time_base())?;
		
		let framerate = decoder.frame_rate().unwrap_or(Rational::new(60, 1));
		let rate_time_base = framerate.invert();
		
		let encoder = params.backend.create_encoder(
			params.output_codec,
			rate_time_base,
			decoder.width() * 1080 / decoder.height(),
			1080,
			Some(framerate),
			params.bit_rate,
			has_global_header,
			params.encoder_options
		).context("Creating encoder")?;
		
		println!("Video decoder: {}", decoder.codec().unwrap().description());
		
		let mut out_stream = params.muxer.add_stream(encoder.codec())?;
		out_stream.set_parameters(&encoder);
		out_stream.set_time_base(rate_time_base);
		
		// let rescaler = ffmpeg::software::converter((decoder.width(), decoder.height()), decoder.format(), DEFAULT_PIXEL_FORMAT)?;
		
		let mut filter = filter::graph::Graph::new();
		let in_params = format!("width={}:height={}:pix_fmt={}:time_base={}/{}:sar=1",
							    decoder.width(),
							    decoder.height(),
							    decoder.format().descriptor().unwrap().name(),
		                        rate_time_base.numerator(), rate_time_base.denominator()
		);
		
		filter.add(&filter::find("buffer").unwrap(), "in", &in_params)?;
		filter.add(&filter::find("buffersink").unwrap(), "out", "")?;
		
		{
			let mut out = filter.get("out").unwrap();
			out.set_pixel_format(encoder.format());
		}
		
		let filter_spec = "scale=w=-2:h=1080";
		
		filter.output("in", 0)?.input("out", 0)?.parse(&filter_spec)?;
		filter.validate()?;
		
		Ok(Self {
			decoder,
			encoder,
			filter,
			
			out_stream_index: out_stream.index(),
			rate_time_base,
			first_frame: true,
			last_dts: i64::MIN,
		})
	}
	
	pub fn receive_input_packet(&mut self, in_stream: &ffmpeg::Stream, mut in_packet: Packet, muxer: &mut format::context::Output, time_bounds: Range<i64>) -> anyhow::Result<()> {
		in_packet.rescale_ts(in_stream.time_base(), self.rate_time_base);
		self.decoder.send_packet(&in_packet)?;
		
		self.decode_frames(muxer, time_bounds)
	}
	
	pub fn send_eof(&mut self, muxer: &mut format::context::Output, time_bounds: Range<i64>) -> anyhow::Result<()> {
		self.decoder.send_eof()?;
		self.decode_frames(muxer, time_bounds.clone())?;
		
		self.filter.get("in").unwrap().source().flush()?;
		self.drain_filter(muxer, time_bounds.clone())?;
		
		self.encoder.send_eof()?;
		self.process_output_packets(muxer, time_bounds)?;
		
		Ok(())
	}
	
	fn decode_frames(&mut self, muxer: &mut format::context::Output, time_bounds: Range<i64>) -> anyhow::Result<()> {
		let mut in_frame = frame::Video::empty();
		
		while self.decoder.receive_frame(&mut in_frame).is_ok() {
			let timestamp = in_frame.timestamp()
				.ok_or_else(|| anyhow!("Decoded frame doesn't have a timestamp"))?;
			
			in_frame.set_color_range(ffmpeg::color::Range::MPEG);
			in_frame.set_pts(Some(timestamp));
			
			self.filter.get("in").unwrap().source().add(&in_frame)?;
			self.drain_filter(muxer, time_bounds.clone())?;
		}
		
		Ok(())
	}
	
	fn drain_filter(&mut self, muxer: &mut format::context::Output, time_bounds: Range<i64>) -> anyhow::Result<()> {
		let scaled_time_bounds = Range {
			start: time_bounds.start.rescale((1, 1), self.rate_time_base),
			end: time_bounds.end.rescale((1, 1), self.rate_time_base),
		};
		
		let mut out_frame = frame::Video::empty();
		
		while self.filter.get("out").unwrap().sink().frame(&mut out_frame).is_ok() {
			let pts = out_frame.pts().unwrap();
			
			if pts < scaled_time_bounds.end {
				if scaled_time_bounds.contains(&pts) && self.first_frame {
					out_frame.set_kind(picture::Type::I);
					self.first_frame = false;
				} else {
					out_frame.set_kind(picture::Type::None);
				}
				
				self.encoder.send_frame(&out_frame)?;
				self.process_output_packets(muxer, time_bounds.clone())?;
			}
		}
		
		Ok(())
	}
	
	fn process_output_packets(&mut self, muxer: &mut format::context::Output, time_bounds: Range<i64>) -> anyhow::Result<()> {
		// let stream_time_base = muxer.stream(self.out_stream_index).expect("Unknown stream").time_base();
		
		let scaled_time_bounds = Range {
			start: time_bounds.start.rescale((1, 1), self.rate_time_base),
			end: time_bounds.end.rescale((1, 1), self.rate_time_base),
		};
		
		let mut out_packet = Packet::empty();
		
		while self.encoder.receive_packet(&mut out_packet).is_ok() {
			// out_packet.rescale_ts(self.rate_time_base, stream_time_base);
			
			let pts = out_packet.pts().unwrap();
			
			if scaled_time_bounds.contains(&pts) {
				let mut dts = out_packet.dts().unwrap();
				
				if !scaled_time_bounds.contains(&dts) {
					println!("Out of bounds DTS: {} (pts: {}, begin: {})", dts, pts, scaled_time_bounds.start);
				}
				
				if dts < scaled_time_bounds.start {
					dts = scaled_time_bounds.start;
				}
				
				if dts <= self.last_dts {
					dts = self.last_dts + 1;
				}
				
				self.last_dts = dts;
				out_packet.set_dts(Some(dts));
				
				if pts < dts {
					out_packet.set_pts(Some(dts));
				}
				
				if !scaled_time_bounds.contains(&dts) {
					println!("DTS is still out of bounds! {} ({:?})", dts, scaled_time_bounds);
				}
				
				out_packet.set_stream(self.out_stream_index);
				
				let stream_time_base = muxer.stream(self.out_stream_index).expect("Unknown stream").time_base();
				out_packet.rescale_ts(self.rate_time_base, stream_time_base);
				out_packet.write_interleaved(muxer).context("Writing packet")?;
			}
		}
		
		Ok(())
	}
}
