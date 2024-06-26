use std::ops::Range;

use anyhow::{anyhow, Context};
use ffmpeg_next as ffmpeg;
use ffmpeg_next::{codec, Dictionary, filter, format, frame, Packet, picture, Rational, Rescale};
use ffmpeg_sys_next::{av_buffersrc_parameters_alloc, av_buffersrc_parameters_set, av_free, AVColorRange, AVColorSpace, AVPixelFormat};
use tracing::debug;

use crate::media_manipulation::backends::{VideoBackend, VideoDecoderParams, VideoEncoderParams};
use crate::media_manipulation::media_utils;
use crate::media_manipulation::media_utils::{check_alloc, SECONDS_TIME_BASE};
use crate::media_manipulation::media_utils::av_error;

pub struct VideoTranscoder {
	decoder: codec::decoder::Video,
	encoder: Option<codec::encoder::Video>,
	filter: Option<filter::graph::Graph>,
	
	in_stream_time_base: Rational,
	rate_time_base: Rational,
	first_frame: bool,
	
	backend: Box<dyn VideoBackend>,
	has_global_header: bool,
	output_codec: codec::Id,
	output_width: u32,
	output_height: u32,
	output_framerate: Rational,
	bit_rate: usize,
	encoder_options: Dictionary<'static>,
	
	output_packet_queue: Vec<Packet>,
	out_stream_index: Option<usize>,
}

pub struct VideoTranscoderParams<'a> {
	pub in_stream: &'a ffmpeg::Stream<'a>,
	pub muxer: &'a mut format::context::Output,
	pub backend: Box<dyn VideoBackend>,
	pub output_codec: codec::Id,
	pub target_height: u32,
	pub bit_rate: usize,
	pub encoder_options: Dictionary<'static>,
}

impl VideoTranscoder {
	pub fn new(mut params: VideoTranscoderParams) -> anyhow::Result<Self> {
		let decoder = params.backend.create_decoder(VideoDecoderParams {
			stream_params: params.in_stream.parameters(),
			packet_time_base: params.in_stream.time_base(),
		})?;
		
		let framerate = decoder.frame_rate().unwrap_or(Rational(60, 1));
		let rate_time_base = framerate.invert() * Rational(1, 10);
		
		let output_height = decoder.height().min(params.target_height);
		let output_width = decoder.width() * output_height / decoder.height() / 2 * 2;
		
		let has_global_header = params.muxer.format().flags().contains(format::flag::Flags::GLOBAL_HEADER);
		
		// println!("Video decoder: {}, format: {:?}", decoder.codec().unwrap().description(), decoder.format());
		
		Ok(Self {
			decoder,
			encoder: None,
			filter: None,
			
			in_stream_time_base: params.in_stream.time_base(),
			rate_time_base,
			first_frame: true,
			
			backend: params.backend,
			has_global_header,
			output_codec: params.output_codec,
			output_width,
			output_height,
			output_framerate: framerate,
			bit_rate: params.bit_rate,
			encoder_options: params.encoder_options.to_owned(),
			
			output_packet_queue: Vec::new(),
			out_stream_index: None,
		})
	}
	
	pub fn receive_input_packet(&mut self, in_stream: &ffmpeg::Stream, mut in_packet: Packet, time_bounds: Range<i64>) -> anyhow::Result<()> {
		in_packet.rescale_ts(in_stream.time_base(), self.in_stream_time_base);
		self.decoder.send_packet(&in_packet)?;
		
		self.decode_frames(time_bounds)
	}
	
	pub fn send_eof(&mut self, time_bounds: Range<i64>) -> anyhow::Result<()> {
		self.decoder.send_eof()?;
		self.decode_frames(time_bounds.clone())?;
		
		if let Some(filter) = &mut self.filter {
			filter.get("in").unwrap().source().flush()?;
			self.drain_filter(time_bounds.clone())?;
		}
		
		if let Some(encoder) = &mut self.encoder {
			encoder.send_eof()?;
			self.process_output_packets(time_bounds.clone())?;
		}
		
		Ok(())
	}
	
	fn decode_frames(&mut self, time_bounds: Range<i64>) -> anyhow::Result<()> {
		let scaled_time_bounds = media_utils::scale_range(time_bounds.clone(), SECONDS_TIME_BASE, self.rate_time_base);
		let mut in_frame = frame::Video::empty();
		
		while self.decoder.receive_frame(&mut in_frame).is_ok() {
			let timestamp = in_frame.timestamp()
				.ok_or_else(|| anyhow!("Decoded frame doesn't have a timestamp"))?
				.rescale(self.in_stream_time_base, self.rate_time_base);
			
			in_frame.set_pts(Some(timestamp));
			
			// Only pass frames in the time bounds to the filter
			if scaled_time_bounds.contains(&timestamp) {
				if self.filter.is_none() {
					let pixel_format: AVPixelFormat = in_frame.format().into();
					let color_space: AVColorSpace = self.decoder.color_space().into();
					let color_range: AVColorRange = self.decoder.color_range().into();
					
					let mut filter = filter::graph::Graph::new();
					
					let in_params = format!("width={}:height={}:pix_fmt={}:time_base={}/{}:sar=1:colorspace={}:range={}",
						self.decoder.width(), self.decoder.height(),
						pixel_format as u32,
						self.rate_time_base.numerator(), self.rate_time_base.denominator(),
						color_space as u32,
						color_range as u32,
					);
					
					unsafe {
						let mut in_filter = filter.add(&filter::find("buffer").unwrap(), "in", &in_params)?;
						let hw_frames_ctx = (*self.decoder.as_ptr()).hw_frames_ctx;
						
						let par = check_alloc(av_buffersrc_parameters_alloc())?;
						let mut par = scopeguard::guard(par, |ptr| av_free(ptr.cast()));
						
						if !hw_frames_ctx.is_null() {
							(**par).hw_frames_ctx = hw_frames_ctx;
						}
						
						av_error(av_buffersrc_parameters_set(in_filter.as_mut_ptr(), *par))
							.context("Setting buffersrc filter params")?;
					}
					
					{
						let mut out_filter = filter.add(&filter::find("buffersink").unwrap(), "out", "")?;
						out_filter.set_pixel_format(self.backend.encoder_pixel_format());
					}
					
					let filter_spec = self.backend.create_filter_chain(self.output_width, self.output_height);
					
					filter.output("in", 0)?.input("out", 0)?.parse(&filter_spec)?;
					filter.validate()?;
					
					self.filter = Some(filter);
				}
				
				self.filter.as_mut().unwrap().get("in").unwrap().source().add(&in_frame)?;
				self.drain_filter(time_bounds.clone())?;
			}
		}
		
		Ok(())
	}
	
	fn drain_filter(&mut self, time_bounds: Range<i64>) -> anyhow::Result<()> {
		let scaled_time_bounds = media_utils::scale_range(time_bounds.clone(), SECONDS_TIME_BASE, self.rate_time_base);
		let mut out_frame = frame::Video::empty();
		
		while self.filter.as_mut().unwrap().get("out").unwrap().sink().frame(&mut out_frame).is_ok() {
			if self.encoder.is_none() {
				let hw_ctx = unsafe { (*out_frame.as_ptr()).hw_frames_ctx };
				
				let encoder = self.backend.create_encoder(VideoEncoderParams {
					codec: self.output_codec,
					global_header: self.has_global_header,
					time_base: self.rate_time_base,
					width: self.output_width,
					height: self.output_height,
					framerate: Some(self.output_framerate),
					bitrate: self.bit_rate,
					encoder_options: self.encoder_options.clone(),
					input_hw_ctx: Some(hw_ctx),
				}).context("Creating encoder")?;
				
				self.encoder = Some(encoder);
			}
			
			let pts = out_frame.pts().unwrap();
			
			// Only pass frames in the time bounds to the encoder
			if scaled_time_bounds.contains(&pts) {
				// Make the first frame an Iframe (probably unnecessary)
				if self.first_frame {
					out_frame.set_kind(picture::Type::I);
					self.first_frame = false;
				} else {
					out_frame.set_kind(picture::Type::None);
				}
				
				// if out_frame.color_range() == ffmpeg::color::Range::Unspecified {
				// 	out_frame.set_color_range(ffmpeg::color::Range::JPEG);
				// }
				
				self.encoder.as_mut().unwrap().send_frame(&out_frame)?;
				self.process_output_packets(time_bounds.clone())?;
			}
		}
		
		Ok(())
	}
	
	fn process_output_packets(&mut self, time_bounds: Range<i64>) -> anyhow::Result<()> {
		let scaled_time_bounds = media_utils::scale_range(time_bounds.clone(), SECONDS_TIME_BASE, self.rate_time_base);
		let mut out_packet = Packet::empty();
		
		while self.encoder.as_mut().unwrap().receive_packet(&mut out_packet).is_ok() {
			let pts = out_packet.pts().unwrap();
			
			// Again, only write packets inside the time bounds
			if scaled_time_bounds.contains(&pts) {
				let dts = out_packet.dts().unwrap();
				
				if !scaled_time_bounds.contains(&dts) {
					debug!("Out of bounds DTS: {} (pts: {}, begin: {})", dts, pts, scaled_time_bounds.start);
				}
				
				self.output_packet_queue.push(out_packet.clone());
			}
		}
		
		Ok(())
	}
	
	pub fn add_output_stream(&mut self, muxer: &mut format::context::Output) -> anyhow::Result<()> {
		if let Some(encoder) = &self.encoder {
			let mut out_stream = muxer.add_stream(self.output_codec)?;
			out_stream.set_parameters(encoder);
			out_stream.set_time_base(self.rate_time_base);
			
			self.out_stream_index = Some(out_stream.index());
		}
		
		Ok(())
	}
	
	pub fn write_output_packets(&mut self, muxer: &mut format::context::Output) -> anyhow::Result<()> {
		let Some(out_stream_index) = self.out_stream_index else { return Ok(()); };
		let stream_time_base = muxer.stream(out_stream_index).expect("Unknown stream").time_base();
		
		for mut out_packet in self.output_packet_queue.drain(..) {
			out_packet.set_stream(out_stream_index);
			out_packet.rescale_ts(self.rate_time_base, stream_time_base);
			
			out_packet.write_interleaved(muxer).context("Writing packet")?;
		}
		
		Ok(())
	}
}
