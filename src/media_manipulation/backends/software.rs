use anyhow::{anyhow, Context};
use ffmpeg_next::{codec, decoder, Dictionary, encoder, Rational};
use ffmpeg_next::codec::Parameters;
use ffmpeg_next::format::Pixel;

use crate::media_manipulation::backends::VideoBackend;

pub struct SoftwareVideoBackend;

impl SoftwareVideoBackend {
	pub fn new() -> Self {
		Self
	}
}

impl VideoBackend for SoftwareVideoBackend {
	fn create_encoder(
		&mut self,
		codec: codec::Id,
		time_base: Rational,
		width: u32,
		height: u32,
		framerate: Option<Rational>,
		bitrate: usize,
		global_header: bool,
		mut encoder_options: Dictionary
	) -> anyhow::Result<encoder::Video> {
		let encoder_name = match codec {
			codec::Id::H264 => {
				encoder_options.set("preset", "veryfast");
				encoder_options.set("profile", "high");
				encoder_options.set("forced-idr", "1");
				
				"libx264"
			},
			_ => return Err(anyhow!("Unsupported encoder codec"))
		};
		
		let encoder_codec = encoder::find_by_name(encoder_name)
			.ok_or_else(|| anyhow!("Unable to find encoder"))?;
		
		let mut encoder = codec::context::Context::new_with_codec(encoder_codec)
			.encoder()
			.video()?;
		
		if global_header {
			encoder.set_flags(codec::flag::Flags::GLOBAL_HEADER);
		}
		
		encoder.set_time_base(time_base);
		encoder.set_width(width);
		encoder.set_height(height);
		encoder.set_format(Pixel::YUV420P);
		encoder.set_frame_rate(framerate);
		encoder.set_bit_rate(bitrate);
		
		encoder.open_as_with(encoder_codec, encoder_options).context("Opening encoder")
	}
	
	fn create_decoder(&mut self, params: Parameters, packet_time_base: Rational) -> anyhow::Result<decoder::Video> {
		let decoder_context = codec::context::Context::from_parameters(params)?;
		let mut decoder = decoder_context.decoder().video().context("Opening decoder")?;
		
		decoder.set_packet_time_base(packet_time_base);
		
		Ok(decoder)
	}
}