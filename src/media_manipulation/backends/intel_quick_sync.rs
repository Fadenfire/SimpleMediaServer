use anyhow::{anyhow, Context};
use ffmpeg_next::{codec, decoder, encoder, Rational};
use ffmpeg_next::codec::Parameters;
use ffmpeg_next::format::Pixel;
use ffmpeg_sys_next::{av_buffer_ref, AVCodecContext, AVPixelFormat};
use ffmpeg_sys_next::AVHWDeviceType::AV_HWDEVICE_TYPE_QSV;
use ffmpeg_sys_next::AVPixelFormat::{AV_PIX_FMT_NONE, AV_PIX_FMT_QSV};

use crate::media_manipulation::backends::{VideoBackend, VideoEncoderParams};
use crate::media_manipulation::utils::hardware_device::HardwareDeviceContext;

pub struct QuickSyncVideoBackend {
	hw_context: HardwareDeviceContext,
}

impl QuickSyncVideoBackend {
	pub fn new() -> anyhow::Result<Self> {
		let hw_context = HardwareDeviceContext::create(AV_HWDEVICE_TYPE_QSV)
			.context("Creating QS device")?;
		
		Ok(Self {
			hw_context
		})
	}
	
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
			if *formats == AV_PIX_FMT_QSV {
				return AV_PIX_FMT_QSV;
			}
			
			formats = formats.add(1);
		}
		
		tracing::error!("The QSV pixel format is not offered in get_format()");
		
		return AV_PIX_FMT_NONE;
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
		params.encoder_options.set("look_ahead", "1");
		
		let mut encoder = codec::context::Context::new_with_codec(encoder_codec)
			.encoder()
			.video()?;
		
		unsafe {
			let hw_ctx = av_buffer_ref(params.input_hw_ctx.expect("Backend requires input HW context"));
			if hw_ctx.is_null() { return Err(anyhow!("Couldn't duplicate HW context")); }
			
			(*encoder.as_mut_ptr()).hw_frames_ctx = hw_ctx;
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
	
	fn create_decoder(&mut self, params: Parameters, packet_time_base: Rational) -> anyhow::Result<decoder::Video> {
		let decoder_name = Self::get_codec_name(params.id());
		
		let decoder_codec = decoder_name
			.and_then(|name| decoder::find_by_name(name))
			.or_else(|| decoder::find(params.id()))
			.ok_or_else(|| anyhow!("Unable to find decoder"))?;
		
		let mut decoder_context = codec::context::Context::new_with_codec(decoder_codec);
		decoder_context.set_parameters(params)?;
		
		unsafe {
			let ctx = decoder_context.as_mut_ptr();
			(*ctx).hw_device_ctx = self.hw_context.add_ref()?;
			(*ctx).get_format = Some(Self::get_format);
			(*ctx).pkt_timebase = packet_time_base.into();
		}
		
		let decoder = decoder_context.decoder().video().context("Opening decoder")?;
		
		Ok(decoder)
	}
	
	fn create_framerate_filter(&self, framerate: u32) -> String {
		format!("vpp_qsv=framerate={}", framerate)
	}
	
	fn create_scaling_filter(&self, width: u32, height: u32) -> String {
		format!("vpp_qsv=w={}:h={}", width, height)
	}
}