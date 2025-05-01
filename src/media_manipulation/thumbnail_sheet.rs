use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use bytes::Bytes;
use ffmpeg_next::{decoder, format, frame, media, rescale, Rescale};
use ffmpeg_sys_next::AV_CODEC_FLAG_COPY_OPAQUE;
use image::{GenericImage, Rgb, RgbImage};
use serde::{Deserialize, Serialize};
use turbojpeg::Subsamp;

use crate::media_manipulation::backends::{BackendFactory, VideoDecoderParams};
use crate::media_manipulation::media_utils;
use crate::media_manipulation::media_utils::frame_scaler::FrameScaler;
use crate::media_manipulation::media_utils::{MILLIS_TIME_BASE, SECONDS_TIME_BASE};

const TARGET_THUMBNAIL_HEIGHT: u32 = 120;
const JPEG_QUALITY: i32 = 90;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThumbnailSheetParams {
	pub thumbnail_width: u32,
	pub thumbnail_height: u32,
	pub thumbnail_count: u32,
	pub sheet_rows: u32,
	pub sheet_cols: u32,
	pub interval: u32,
}

pub fn calculate_sheet_params(video_duration: Duration, video_width: u32, video_height: u32) -> ThumbnailSheetParams {
	let interval = (video_duration.as_secs() / 500).max(5) as u32;
	
	let thumbnail_count = (video_duration.as_secs_f64() / interval as f64).ceil() as u32;
	let sheet_dimension = (thumbnail_count as f64).sqrt().ceil() as u32;
	
	ThumbnailSheetParams {
		thumbnail_width: video_width * TARGET_THUMBNAIL_HEIGHT / video_height,
		thumbnail_height: TARGET_THUMBNAIL_HEIGHT,
		thumbnail_count,
		sheet_rows: sheet_dimension,
		sheet_cols: sheet_dimension,
		interval,
	}
}

pub fn generate_sheet(backend_factory: &impl BackendFactory, media_path: PathBuf) -> anyhow::Result<(Bytes, ThumbnailSheetParams)> {
	let mut demuxer = format::input(&media_path).context("Opening video file")?;
	
	let video_stream = demuxer.streams().best(media::Type::Video).unwrap();
	let video_stream_index = video_stream.index();
	
	let mut video_backend = backend_factory.create_video_backend().context("Creating video backend")?;
	
	let mut decoder = video_backend.create_decoder(VideoDecoderParams {
		stream_params: video_stream.parameters(),
		packet_time_base: video_stream.time_base(),
		flags: AV_CODEC_FLAG_COPY_OPAQUE,
		..Default::default()
	})?;
	
	media_utils::discard_all_but_keyframes(&mut demuxer, video_stream_index);
	
	let duration_millis = demuxer.duration()
		.rescale(rescale::TIME_BASE, MILLIS_TIME_BASE)
		.try_into()
		.context("Duration is negative")?;
	
	let duration = Duration::from_millis(duration_millis);
	let sheet_params = calculate_sheet_params(duration, decoder.width(), decoder.height());
	
	let mut sprite_sheet = RgbImage::new(
		sheet_params.sheet_cols * sheet_params.thumbnail_width,
		sheet_params.sheet_rows * sheet_params.thumbnail_height,
	);
	
	let mut scaler = FrameScaler::new();
	let mut frame = frame::Video::empty();
	
	let mut receive_frames = |decoder: &mut decoder::Video| -> anyhow::Result<()> {
		while decoder.receive_frame(&mut frame).is_ok() {
			let Some(frame_opaque) = media_utils::get_frame_opaque(&frame) else { continue };
			let offset = (frame_opaque.get() - 1) as u32;
			
			let rgb_frame = scaler.scale_frame_rgb(&frame, sheet_params.thumbnail_width, sheet_params.thumbnail_height)?;
			let image_view = media_utils::frame_image_sample_rgb(&rgb_frame);
			
			let x_pos = (offset % sheet_params.sheet_cols) * sheet_params.thumbnail_width;
			let y_pos = (offset / sheet_params.sheet_rows) * sheet_params.thumbnail_height;
			
			sprite_sheet.copy_from(&image_view.as_view::<Rgb<u8>>().unwrap(), x_pos, y_pos).unwrap();
		}
		
		Ok(())
	};
	
	for offset in 0..sheet_params.thumbnail_count {
		let time = (offset * sheet_params.interval).rescale(SECONDS_TIME_BASE, rescale::TIME_BASE);
		
		if demuxer.seek(time, time..).is_err() { continue; }
		
		// Add one to make it one indexed
		let packet_opaque = NonZeroUsize::new(offset as usize + 1).unwrap();
		
		media_utils::push_one_packet(&mut demuxer, &mut decoder, video_stream_index, Some(packet_opaque))?;
		receive_frames(&mut decoder)?;
	}
	
	decoder.send_eof()?;
	receive_frames(&mut decoder)?;
	
	let output_buffer = turbojpeg::compress_image(&sprite_sheet, JPEG_QUALITY, Subsamp::Sub2x2).unwrap();
	
	Ok((Bytes::from(output_buffer.to_vec()), sheet_params))
}