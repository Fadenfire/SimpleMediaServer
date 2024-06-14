use ffmpeg_next::{codec, Dictionary, Rational};
use ffmpeg_next::format::Pixel;
use ffmpeg_sys_next::AVBufferRef;

pub mod software;
pub mod video_toolbox;
pub mod intel_quick_sync;

pub struct VideoEncoderParams<'a> {
	pub codec: codec::Id,
	pub global_header: bool,
	pub time_base: Rational,
	pub width: u32,
	pub height: u32,
	pub framerate: Option<Rational>,
	pub bitrate: usize,
	pub encoder_options: Dictionary<'a>,
	pub input_hw_ctx: Option<*const AVBufferRef>,
}

pub trait VideoBackend {
	fn encoder_pixel_format(&self) -> Pixel;
	
	fn create_encoder(&mut self, params: VideoEncoderParams) -> anyhow::Result<codec::encoder::Video>;
	
	fn create_decoder(
		&mut self,
		params: codec::Parameters,
		packet_time_base: Rational,
	) -> anyhow::Result<codec::decoder::Video>;
	
	fn create_framerate_filter(&self, framerate: u32) -> String {
		format!("framerate=fps={}", framerate)
	}
	
	fn create_scaling_filter(&self, width: u32, height: u32) -> String {
		format!("scale=w={}:h={}", width, height)
	}
}

pub trait BackendFactory {
	fn create_video_backend(&self) -> anyhow::Result<Box<impl VideoBackend + 'static>>;
}
