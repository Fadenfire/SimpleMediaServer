use std::ffi::c_uint;

use ffmpeg_next::{codec, Dictionary, Rational};
use ffmpeg_next::format::Pixel;
use ffmpeg_sys_next::AVBufferRef;

pub mod software;
pub mod video_toolbox;
pub mod intel_quick_sync;

#[non_exhaustive]
pub struct VideoEncoderParams {
	pub codec: codec::Id,
	pub global_header: bool,
	pub time_base: Rational,
	pub width: u32,
	pub height: u32,
	pub framerate: Option<Rational>,
	pub bitrate: usize,
	pub encoder_options: Dictionary<'static>,
	pub input_hw_ctx: Option<*const AVBufferRef>,
}

impl Default for VideoEncoderParams {
	fn default() -> Self {
		Self {
			codec: codec::Id::None,
			global_header: false,
			time_base: Rational(1, 0),
			width: 0,
			height: 0,
			framerate: None,
			bitrate: 0,
			encoder_options: Dictionary::new(),
			input_hw_ctx: None,
		}
	}
}

#[non_exhaustive]
pub struct VideoDecoderParams {
	pub stream_params: codec::Parameters,
	pub packet_time_base: Rational,
	pub flags: c_uint,
}

impl Default for VideoDecoderParams {
	fn default() -> Self {
		Self {
			stream_params: codec::Parameters::default(),
			packet_time_base: Rational(1, 0),
			flags: 0,
		}
	}
}

pub trait VideoBackend {
	fn encoder_pixel_format(&self) -> Pixel;
	
	fn create_encoder(&mut self, params: VideoEncoderParams) -> anyhow::Result<codec::encoder::Video>;
	
	fn create_decoder(&mut self, params: VideoDecoderParams) -> anyhow::Result<codec::decoder::Video>;
	
	fn create_filter_chain(&self, width: u32, height: u32) -> String {
		format!("scale=w={}:h={}", width, height)
	}
}

pub trait BackendFactory {
	fn create_video_backend(&self) -> anyhow::Result<Box<dyn VideoBackend>>;
}
