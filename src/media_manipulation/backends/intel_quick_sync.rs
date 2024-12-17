use std::ffi::c_int;
use std::sync::Arc;
use anyhow::{anyhow, Context};
use ffmpeg_next::{codec, decoder, encoder};
use ffmpeg_next::format::Pixel;
use ffmpeg_sys_next::{av_buffer_ref, AVCodecContext, AVPixelFormat};
use ffmpeg_sys_next::AVHWDeviceType::AV_HWDEVICE_TYPE_VAAPI;
use ffmpeg_sys_next::AVPixelFormat::{AV_PIX_FMT_NONE, AV_PIX_FMT_VAAPI};
use tracing::info;
use crate::media_manipulation::backends::{BackendFactory, VideoBackend, VideoDecoderParams, VideoEncoderParams};
use crate::media_manipulation::media_utils::check_alloc;
use crate::media_manipulation::media_utils::hardware_device::{BorrowedDevice, DevicePool, HardwareDeviceContext};

pub struct QuickSyncVideoBackendFactory {
	device_pool: Arc<DevicePool>,
}

impl QuickSyncVideoBackendFactory {
	pub fn new() -> Self {
		Self {
			device_pool: DevicePool::new(|| {
				info!("Creating new QSV device for the pool");
				
				HardwareDeviceContext::create(AV_HWDEVICE_TYPE_VAAPI).map_err(Into::into)
			}),
		}
	}
}

impl BackendFactory for QuickSyncVideoBackendFactory {
	fn create_video_backend(&self) -> anyhow::Result<Box<dyn VideoBackend>> {
		Ok(Box::new(QuickSyncVideoBackend {
			hw_context: self.device_pool.take_device()?,
		}))
	}
}

pub struct QuickSyncVideoBackend {
	hw_context: BorrowedDevice,
}

impl QuickSyncVideoBackend {
	fn get_codec_name(codec: codec::Id) -> Option<&'static str> {
		match codec {
			codec::Id::H264 => Some("h264_qsv"),
			codec::Id::HEVC => Some("hevc_qsv"),
			codec::Id::VP9 => Some("vp9_qsv"),
			codec::Id::VP8 => Some("vp8_qsv"),
			codec::Id::AV1 => Some("av1_qsv"),
			codec::Id::MPEG2VIDEO => Some("mpeg2_qsv"),
			codec::Id::MJPEG => Some("mjpeg_qsv"),
			_ => None
		}
	}
	
	unsafe extern "C" fn get_format(_ctx: *mut AVCodecContext, mut formats: *const AVPixelFormat) -> AVPixelFormat {
		while *formats != AV_PIX_FMT_NONE {
			if *formats == AV_PIX_FMT_VAAPI {
				return AV_PIX_FMT_VAAPI;
			}
			
			formats = formats.add(1);
		}
		
		tracing::error!("The VA API pixel format is not offered in get_format()");
		
		AV_PIX_FMT_NONE
	}
}

impl VideoBackend for QuickSyncVideoBackend {
	fn encoder_pixel_format(&self) -> Pixel {
		Pixel::QSV
	}
	
	fn create_encoder(&mut self, mut params: VideoEncoderParams) -> anyhow::Result<encoder::Video> {
		let encoder_name = Self::get_codec_name(params.codec)
			.ok_or_else(|| anyhow!("Unsupported encoder codec"))?;
		
		let encoder_codec = encoder::find_by_name(encoder_name)
			.ok_or_else(|| anyhow!("Unable to find encoder"))?;
		
		params.encoder_options.set("low_power", "1");
		params.encoder_options.set("look_ahead", "0");
		
		let mut encoder = codec::context::Context::new_with_codec(encoder_codec)
			.encoder()
			.video()?;
		
		unsafe {
			let hw_frames_ctx = params.input_hw_ctx
				.filter(|p| !p.is_null())
				.expect("Backend requires input HW context");
			
			(*encoder.as_mut_ptr()).hw_frames_ctx = check_alloc(av_buffer_ref(hw_frames_ctx))?;
		}
		
		if params.global_header {
			encoder.set_flags(codec::flag::Flags::GLOBAL_HEADER);
		}
		
		encoder.set_time_base(params.time_base);
		encoder.set_width(params.width);
		encoder.set_height(params.height);
		encoder.set_format(Pixel::QSV);
		encoder.set_frame_rate(params.framerate);
		encoder.set_bit_rate(params.bitrate);
		
		encoder.open_as_with(encoder_codec, params.encoder_options).context("Opening encoder")
	}
	
	fn create_decoder(&mut self, params: VideoDecoderParams) -> anyhow::Result<decoder::Video> {
		let mut decoder_context = codec::context::Context::from_parameters(params.stream_params)?;
		
		unsafe {
			let ctx = decoder_context.as_mut_ptr();
			(*ctx).hw_device_ctx = self.hw_context.add_ref()?;
			(*ctx).get_format = Some(Self::get_format);
			(*ctx).pkt_timebase = params.packet_time_base.into();
			(*ctx).flags |= params.flags as c_int;
		}
		
		let decoder = decoder_context.decoder().video().context("Opening decoder")?;
		
		Ok(decoder)
	}
	
	fn create_filter_chain(&self, width: u32, height: u32) -> String {
		format!("scale_vaapi=w={}:h={}:format=nv12:extra_hw_frames=24,hwmap=derive_device=qsv,format=qsv", width, height)
	}
}