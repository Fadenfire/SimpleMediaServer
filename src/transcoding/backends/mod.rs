pub mod software;

use ffmpeg_the_third::{codec, Dictionary, Rational};

pub trait VideoBackend {
	fn create_encoder(
		&mut self,
		codec: codec::Id,
		time_base: Rational,
		width: u32,
		height: u32,
		framerate: Option<Rational>,
		bitrate: usize,
		global_header: bool,
		encoder_options: Dictionary
	) -> anyhow::Result<codec::encoder::Video>;
	
	fn create_decoder(
		&mut self,
		params: codec::Parameters
	) -> anyhow::Result<codec::decoder::Video>;
}