use anyhow::{anyhow, Context};
use ffmpeg_sys_the_third::avcodec_alloc_context3;
use ffmpeg_the_third::{codec, decoder, Dictionary, encoder, Rational};
use ffmpeg_the_third::codec::Parameters;
use ffmpeg_the_third::format::Pixel;

use crate::media_manipulation::backends::VideoBackend;

pub struct VideoToolboxVideoBackend;

impl VideoToolboxVideoBackend {
	pub fn new() -> Self {
		Self
	}
}

impl VideoBackend for VideoToolboxVideoBackend {
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
	) -> anyhow::Result<encoder::Video> {
		let encoder_name = match codec {
			codec::Id::H264 => "h264_videotoolbox",
			_ => return Err(anyhow!("Unsupported encoder codec"))
		};
		
		let encoder_codec = encoder::find_by_name(encoder_name)
			.ok_or_else(|| anyhow!("Unable to find encoder"))
			.and_then(|codec| codec.video().context("Selected encoder doesn't support video"))?;
		
		let encoder_context = unsafe { codec::context::Context::wrap(avcodec_alloc_context3(encoder_codec.as_ptr()), None) };
		let mut encoder = encoder_context.encoder().video()?;
		
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
	
	fn create_decoder(&mut self, params: Parameters) -> anyhow::Result<decoder::Video> {
		let decoder_name = match params.id() {
			codec::Id::H264 => Some("h264_videotoolbox"),
			codec::Id::HEVC => Some("hevc_videotoolbox"),
			codec::Id::VP9 => Some("vp9_videotoolbox"),
			_ => None
		};
		
		let decoder_codec = decoder_name
			.and_then(|name| decoder::find_by_name(name))
			.or_else(|| decoder::find(params.id()))
			.ok_or_else(|| anyhow!("Unable to find decoder"))
			.and_then(|codec| codec.video().context("Selected decoder doesn't support video"))?;
		
		let mut decoder_context = unsafe { codec::context::Context::wrap(avcodec_alloc_context3(decoder_codec.as_ptr()), None) };
		decoder_context.set_parameters(params)?;
		
		decoder_context.decoder()
			.open_as(decoder_codec)
			.context("Opening decoder")?
			.video()
			.context("Decoder doesn't support video")
	}
}