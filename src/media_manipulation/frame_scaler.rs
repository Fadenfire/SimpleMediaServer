use anyhow::Context;
use ffmpeg_next::format::Pixel;
use ffmpeg_next::frame;
use ffmpeg_next::software::scaling;
use ffmpeg_sys_next::av_hwframe_transfer_data;

use crate::media_manipulation::utils::av_error;

pub struct FrameScaler {
	scaler: Option<scaling::Context>,
	software_frame: frame::Video,
	output_frame: frame::Video,
}

impl FrameScaler {
	pub fn new() -> Self {
		Self {
			scaler: None,
			software_frame: frame::Video::empty(),
			output_frame: frame::Video::empty(),
		}
	}
	
	pub fn scale_frame_rgb(
		&mut self,
		in_frame: &frame::Video,
		out_width: u32,
		out_height: u32,
	) -> anyhow::Result<&frame::Video> {
		let mut frame = in_frame;
		
		unsafe {
			if !(*in_frame.as_ptr()).hw_frames_ctx.is_null() {
				if in_frame.width() != self.software_frame.width() || in_frame.height() != self.software_frame.height() {
					self.software_frame = frame::Video::empty();
				}
				
				av_error(av_hwframe_transfer_data(self.software_frame.as_mut_ptr(), in_frame.as_ptr(), 0))
					.context("Transferring HW frame data")?;
				
				frame = &self.software_frame;
			}
		}
		
		let context = match &mut self.scaler {
			Some(ctx) => {
				ctx.cached(
					frame.format(), frame.width(), frame.height(),
					Pixel::RGB24, out_width, out_height,
					scaling::Flags::BICUBIC,
				);
				
				ctx
			}
			scaler_cache => {
				*scaler_cache = Some(scaling::Context::get(
					frame.format(), frame.width(), frame.height(),
					Pixel::RGB24, out_width, out_height,
					scaling::Flags::BICUBIC,
				).context("Creating image converter")?);
				
				scaler_cache.as_mut().unwrap()
			}
		};
		
		if self.output_frame.width() != out_width || self.output_frame.height() != out_height {
			self.output_frame = frame::Video::empty();
		}
		
		context.run(frame, &mut self.output_frame).context("Converting frame")?;
		
		Ok(&self.output_frame)
	}
}

