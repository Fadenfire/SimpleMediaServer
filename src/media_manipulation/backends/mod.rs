use std::ffi::c_uint;

use ffmpeg_next::format::Pixel;
use ffmpeg_next::{codec, color, filter, Dictionary, Rational};
use ffmpeg_sys_next::{AVBufferRef, AVPixelFormat};

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
	pub color_range: color::Range,
	pub color_space: color::Space,
	pub encoder_options: Dictionary<'static>,
	pub input_hw_ctx: Option<*const AVBufferRef>,
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

#[non_exhaustive]
pub struct FilterGraphParams {
	pub output_width: u32,
	pub output_height: u32,
	pub input_pixel_format: AVPixelFormat,
}

pub trait VideoBackend {
	fn encoder_pixel_format(&self) -> Pixel;
	
	fn create_encoder(&mut self, params: VideoEncoderParams) -> anyhow::Result<codec::encoder::Video>;
	
	fn create_decoder(&mut self, params: VideoDecoderParams) -> anyhow::Result<codec::decoder::Video>;
	
	fn build_filter_graph(&self, filter: &mut filter::graph::Graph, params: FilterGraphParams) -> anyhow::Result<()> {
		let filter_spec = format!("scale=w={}:h={}", params.output_width, params.output_height);
		
		filter.output("in", 0)?.input("out", 0)?.parse(&filter_spec)?;
		
		Ok(())
	}
}

pub trait BackendFactory {
	fn create_video_backend(&self) -> anyhow::Result<Box<dyn VideoBackend>>;
	
	fn supports_encoding_codec(&self, codec: codec::Id) -> bool {
		codec == codec::Id::H264
	}
}

fn set_up_video_encoder(encoder: &mut codec::encoder::video::Video, params: &VideoEncoderParams) {
	if params.global_header {
		encoder.set_flags(codec::flag::Flags::GLOBAL_HEADER);
	}
	
	encoder.set_time_base(params.time_base);
	encoder.set_width(params.width);
	encoder.set_height(params.height);
	encoder.set_color_range(params.color_range);
	encoder.set_colorspace(params.color_space);
	encoder.set_frame_rate(params.framerate);
	encoder.set_bit_rate(params.bitrate);
}
