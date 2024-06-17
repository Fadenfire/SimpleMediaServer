use anyhow::{anyhow, Context};
use ffmpeg_next::{codec, decoder, encoder};
use ffmpeg_next::format::Pixel;

use crate::media_manipulation::backends::{VideoBackend, VideoDecoderParams, VideoEncoderParams};

pub struct VideoToolboxVideoBackend;

impl VideoToolboxVideoBackend {
	pub fn new() -> Self {
		Self
	}
}

impl VideoBackend for VideoToolboxVideoBackend {
	fn encoder_pixel_format(&self) -> Pixel {
		Pixel::YUV420P
	}
	
	fn create_encoder(&mut self, params: VideoEncoderParams) -> anyhow::Result<encoder::Video> {
		let encoder_name = match params.codec {
			codec::Id::H264 => "h264_videotoolbox",
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
		let decoder_name = match params.stream_params.id() {
			codec::Id::H264 => Some("h264_videotoolbox"),
			codec::Id::HEVC => Some("hevc_videotoolbox"),
			codec::Id::VP9 => Some("vp9_videotoolbox"),
			_ => None
		};
		
		let decoder_codec = decoder_name
			.and_then(|name| decoder::find_by_name(name))
			.or_else(|| decoder::find(params.stream_params.id()))
			.ok_or_else(|| anyhow!("Unable to find decoder"))?;
		
		let mut decoder_context = codec::context::Context::new_with_codec(decoder_codec);
		decoder_context.set_parameters(params.stream_params)?;
		
		let mut decoder = decoder_context.decoder().video().context("Opening decoder")?;
		
		decoder.set_packet_time_base(params.packet_time_base);
		
		Ok(decoder)
	}
}