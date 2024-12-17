use std::ffi::c_int;
use anyhow::{anyhow, Context};
use ffmpeg_next::{codec, decoder, encoder};
use ffmpeg_next::format::Pixel;

use crate::media_manipulation::backends::{BackendFactory, VideoBackend, VideoDecoderParams, VideoEncoderParams};

pub struct SoftwareVideoBackendFactory;

impl SoftwareVideoBackendFactory {
	pub fn new() -> Self {
		Self
	}
}

impl BackendFactory for SoftwareVideoBackendFactory {
	fn create_video_backend(&self) -> anyhow::Result<Box<dyn VideoBackend>> {
		Ok(Box::new(SoftwareVideoBackend))
	}
}

pub struct SoftwareVideoBackend;

impl VideoBackend for SoftwareVideoBackend {
	fn encoder_pixel_format(&self) -> Pixel {
		Pixel::YUV420P
	}
	
	fn create_encoder(&mut self, mut params: VideoEncoderParams) -> anyhow::Result<encoder::Video> {
		let encoder_name = match params.codec {
			codec::Id::H264 => {
				params.encoder_options.set("preset", "veryfast");
				params.encoder_options.set("profile", "high");
				params.encoder_options.set("forced-idr", "1");
				
				"libx264"
			},
			_ => return Err(anyhow!("Unsupported encoder codec"))
		};
		
		let encoder_codec = encoder::find_by_name(encoder_name)
			.ok_or_else(|| anyhow!("Unable to find encoder"))?;
		
		let mut encoder = codec::context::Context::new_with_codec(encoder_codec)
			.encoder()
			.video()?;
		
		if params.global_header {
			encoder.set_flags(codec::flag::Flags::GLOBAL_HEADER);
		}
		
		encoder.set_time_base(params.time_base);
		encoder.set_width(params.width);
		encoder.set_height(params.height);
		encoder.set_format(Pixel::YUV420P);
		encoder.set_frame_rate(params.framerate);
		encoder.set_bit_rate(params.bitrate);
		
		encoder.open_as_with(encoder_codec, params.encoder_options).context("Opening encoder")
	}
	
	fn create_decoder(&mut self, params: VideoDecoderParams) -> anyhow::Result<decoder::Video> {
		let mut decoder_context = codec::context::Context::from_parameters(params.stream_params)?;
		
		unsafe { (*decoder_context.as_mut_ptr()).flags |= params.flags as c_int; }
		
		let mut decoder = decoder_context.decoder().video().context("Opening decoder")?;
		
		decoder.set_packet_time_base(params.packet_time_base);
		
		Ok(decoder)
	}
}
