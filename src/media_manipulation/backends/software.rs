use anyhow::{anyhow, Context};
use ffmpeg_next::{codec, decoder, encoder, Rational};
use ffmpeg_next::codec::Parameters;
use ffmpeg_next::format::Pixel;

use crate::media_manipulation::backends::{BackendFactory, VideoBackend, VideoEncoderParams};

pub struct SoftwareVideoBackend;

impl SoftwareVideoBackend {
	pub fn new() -> Self {
		Self
	}
}

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
	
	fn create_decoder(&mut self, params: Parameters, packet_time_base: Rational) -> anyhow::Result<decoder::Video> {
		let decoder_context = codec::context::Context::from_parameters(params)?;
		let mut decoder = decoder_context.decoder().video().context("Opening decoder")?;
		
		decoder.set_packet_time_base(packet_time_base);
		
		Ok(decoder)
	}
}

pub struct SoftwareBackendFactory;

impl BackendFactory for SoftwareBackendFactory {
	fn create_video_backend(&self) -> anyhow::Result<Box<impl VideoBackend + 'static>> {
		Ok(Box::new(SoftwareVideoBackend::new()))
	}
}
