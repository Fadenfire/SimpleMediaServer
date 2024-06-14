use std::cmp::min;
use std::ops::Range;

use anyhow::{anyhow, Context};
use ffmpeg_next as ffmpeg;
use ffmpeg_next::{codec, Dictionary, format, frame, Packet, Rational, Rescale};

use crate::media_manipulation::media_utils;
use crate::media_manipulation::media_utils::SECONDS_TIME_BASE;

pub struct AudioTranscoder {
	decoder: codec::decoder::Audio,
	encoder: codec::encoder::Audio,
	
	in_stream_time_base: Rational,
	out_stream_index: usize,
	rate_time_base: Rational,
	sample_size: usize,
	
	staging_frame: frame::Audio,
	staging_index: usize,
	first_frame: bool,
	output_packet_queue: Vec<Packet>,
}

pub struct AudioTranscoderParams<'a> {
	pub in_stream: &'a ffmpeg::Stream<'a>,
	pub muxer: &'a mut format::context::Output,
	pub encoder_codec: codec::Audio,
	pub bit_rate: usize,
	pub encoder_options: Dictionary<'a>,
}

impl AudioTranscoder {
	pub fn new(params: AudioTranscoderParams) -> anyhow::Result<Self> {
		let has_global_header = params.muxer.format().flags().contains(format::flag::Flags::GLOBAL_HEADER);
		
		let mut decoder = codec::context::Context::from_parameters(params.in_stream.parameters())?
			.decoder()
			.audio()?;
		
		decoder.set_packet_time_base(params.in_stream.time_base());
		
		let rate_time_base = Rational::new(1, decoder.rate() as i32);
		
		let mut encoder = codec::context::Context::new_with_codec(*params.encoder_codec)
			.encoder()
			.audio()?;
		
		if has_global_header {
			encoder.set_flags(codec::flag::Flags::GLOBAL_HEADER);
		}
		
		encoder.set_rate(decoder.rate() as i32);
		encoder.set_channel_layout(decoder.channel_layout());
		encoder.set_format(decoder.format());
		encoder.set_bit_rate(params.bit_rate);
		encoder.set_time_base(rate_time_base);
		
		let encoder = encoder.open_with(params.encoder_options)?;
		
		let mut out_stream = params.muxer.add_stream(params.encoder_codec)?;
		out_stream.set_parameters(&encoder);
		out_stream.set_time_base(rate_time_base);
		
		let staging_frame = frame::Audio::new(encoder.format(), encoder.frame_size() as usize, encoder.channel_layout());
		
		let mut sample_size = staging_frame.format().bytes();
		
		if staging_frame.format().is_packed() {
			sample_size *= staging_frame.channel_layout().channels() as usize;
		}
		
		Ok(Self {
			decoder,
			encoder,
			
			in_stream_time_base: params.in_stream.time_base(),
			out_stream_index: out_stream.index(),
			rate_time_base,
			sample_size,
			
			staging_frame,
			staging_index: 0,
			first_frame: true,
			output_packet_queue: Vec::new(),
		})
	}
	
	pub fn receive_input_packet(&mut self, in_stream: &ffmpeg::Stream, mut in_packet: Packet, time_bounds: Range<i64>) -> anyhow::Result<()> {
		in_packet.rescale_ts(in_stream.time_base(), self.in_stream_time_base);
		self.decoder.send_packet(&in_packet).context("Sending packet")?;
		
		self.decode_frames(time_bounds)
	}
	
	pub fn send_eof(&mut self, time_bounds: Range<i64>) -> anyhow::Result<()> {
		self.decoder.send_eof()?;
		self.decode_frames(time_bounds.clone())?;
		
		// If a partial frame remains in the buffer, output it
		if self.staging_index > 0 {
			self.staging_frame.set_samples(self.staging_index);
			
			self.encoder.send_frame(&self.staging_frame)?;
			self.process_output_packets(time_bounds.clone())?;
			
			self.staging_index = 0;
		}
		
		self.encoder.send_eof()?;
		self.process_output_packets(time_bounds.clone())?;
		
		Ok(())
	}
	
	fn decode_frames(&mut self, time_bounds: Range<i64>) -> anyhow::Result<()> {
		let out_frame_size = self.encoder.frame_size() as usize;
		
		let mut in_frame = frame::Audio::empty();
		
		while self.decoder.receive_frame(&mut in_frame).is_ok() {
			let in_frame_size = in_frame.samples();
			
			let timestamp = in_frame.timestamp()
				.ok_or_else(|| anyhow!("Decoded frame doesn't have a timestamp"))?
				.rescale(self.in_stream_time_base, self.rate_time_base);
			
			let mut in_index = 0;
			
			// If this is the start of transcoding, then drop samples until we align to an output frame boundary (times four to be safe)
			if self.first_frame {
				let alignment_size = out_frame_size * 4;
				let correction = (alignment_size - (timestamp % alignment_size as i64) as usize) % alignment_size;
				
				if correction >= in_frame_size {
					continue;
				}
				
				in_index += correction;
				self.first_frame = false;
			}
			
			// Copy samples from input frame to output frame, emitting output frames when they fill up
			while in_index < in_frame_size {
				// If starting a new output frame, set the pts
				if self.staging_index == 0 {
					let out_timestamp = timestamp + in_index as i64;
					self.staging_frame.set_pts(Some(out_timestamp));
				}
				
				let copy_length = min(out_frame_size - self.staging_index, in_frame_size - in_index);
				
				for plane_id in 0..in_frame.planes() {
					unsafe {
						let src = (*in_frame.as_ptr()).data[plane_id];
						let dst = (*self.staging_frame.as_mut_ptr()).data[plane_id];
						
						std::ptr::copy(
							src.add(in_index * self.sample_size),
							dst.add(self.staging_index * self.sample_size),
							copy_length * self.sample_size,
						);
					}
				}
				
				in_index += copy_length;
				self.staging_index += copy_length;
				
				// If output frame is full, send it to the encoder
				if self.staging_index >= out_frame_size {
					self.encoder.send_frame(&self.staging_frame).context("Encoding frame")?;
					self.process_output_packets(time_bounds.clone())?;
					
					self.staging_index = 0;
				}
			}
		}
		
		Ok(())
	}
	
	fn process_output_packets(&mut self, time_bounds: Range<i64>) -> anyhow::Result<()> {
		let scaled_time_bounds = media_utils::scale_range(time_bounds, SECONDS_TIME_BASE, self.rate_time_base);
		let mut out_packet = Packet::empty();
		
		while self.encoder.receive_packet(&mut out_packet).is_ok() {
			let timestamp = out_packet.pts().unwrap();
			
			// Only output frames that lie within the time bounds
			if scaled_time_bounds.contains(&timestamp) {
				self.output_packet_queue.push(out_packet.clone());
			}
		}
		
		Ok(())
	}
	
	pub fn write_output_packets(&mut self, muxer: &mut format::context::Output) -> anyhow::Result<()> {
		let stream_time_base = muxer.stream(self.out_stream_index).expect("Unknown stream").time_base();
		
		for mut out_packet in self.output_packet_queue.drain(..) {
			out_packet.rescale_ts(self.rate_time_base, stream_time_base);
			out_packet.set_stream(self.out_stream_index);
			
			out_packet.write_interleaved(muxer).context("Writing packet")?;
		}
		
		Ok(())
	}
}