use std::path::{Path, PathBuf};

use anyhow::Context;
use bytes::Bytes;
use ffmpeg_the_third as ffmpeg;
use ffmpeg_the_third::{decoder, Discard, format, frame, Rational, Rescale, rescale};
use ffmpeg_the_third::software::scaling;
use image::{GenericImage, Rgb, RgbImage};
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use turbojpeg::Subsamp;

use crate::services::artifact_cache::{ArtifactCache, CacheEntry, FileValidityKey};
use crate::transcoding::backends::software::SoftwareVideoBackend;
use crate::transcoding::backends::VideoBackend;
use crate::transcoding::image_utils::{frame_image_sample_rgb, scale_frame_rgb};

pub struct ThumbnailSheetService {
	cache: ArtifactCache<FileValidityKey, ThumbnailSheetParams>,
	limiter: Semaphore,
}

impl ThumbnailSheetService {
	pub async fn init(cache_dir: PathBuf) -> anyhow::Result<Self> {
		Ok(Self {
			cache: ArtifactCache::<FileValidityKey, ThumbnailSheetParams>::new(cache_dir).await?,
			limiter: Semaphore::new(2),
		})
	}
	
	fn make_cache_key(media_path: &Path) -> String {
		format!("{}.jpg", blake3::hash(media_path.as_os_str().as_encoded_bytes()).to_hex())
	}
	
	pub async fn get_cached_params(&self, media_path: &Path) -> anyhow::Result<Option<ThumbnailSheetParams>> {
		let cache_key = Self::make_cache_key(media_path);
		let validity_key = FileValidityKey::from_file(media_path).await?;
		
		Ok(self.cache.get(&cache_key, &validity_key).await?.map(|entry| entry.metadata))
	}
	
	pub async fn generate_image(&self, media_path: PathBuf) -> anyhow::Result<CacheEntry<ThumbnailSheetParams>> {
		let cache_key = Self::make_cache_key(&media_path);
		let validity_key = FileValidityKey::from_file(&media_path).await?;
		
		self.cache.get_or_insert(&cache_key, validity_key, || async {
			let _permit = self.limiter.acquire().await.unwrap();
			
			tokio::task::spawn_blocking(|| generate_sheet(media_path)).await.unwrap()
		}).await
	}
}

const TARGET_THUMBNAIL_HEIGHT: u32 = 120;
const JPEG_QUALITY: i32 = 90;

const SEC_TIME_BASE: Rational = Rational(1, 1);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThumbnailSheetParams {
	pub thumbnail_width: u32,
	pub thumbnail_height: u32,
	pub thumbnail_count: u32,
	pub sheet_rows: u32,
	pub sheet_cols: u32,
	pub interval: u32,
}

pub fn calculate_sheet_params(duration: i64, video_width: u32, video_height: u32) -> ThumbnailSheetParams {
	let video_duration: u32 = duration.rescale(rescale::TIME_BASE, SEC_TIME_BASE).try_into().unwrap();
	let interval = (video_duration / 500).max(10);
	
	let thumbnail_count = video_duration / interval;
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

pub fn generate_sheet(media_path: PathBuf) -> anyhow::Result<(Bytes, ThumbnailSheetParams)> {
	let mut demuxer = format::input(&media_path).context("Opening video file")?;
	
	let video_stream = demuxer.streams().best(ffmpeg::media::Type::Video).unwrap();
	let video_stream_index = video_stream.index();
	
	let mut video_backend = SoftwareVideoBackend::new();
	let mut decoder = video_backend.create_decoder(video_stream.parameters())?;
	
	decoder.set_parameters(video_stream.parameters())?;
	unsafe { (*decoder.as_mut_ptr()).pkt_timebase = video_stream.time_base().into(); }
	
	for mut stream in demuxer.streams_mut() {
		let discard = if stream.index() == video_stream_index { Discard::NonKey } else { Discard::All };
		unsafe { (*stream.as_mut_ptr()).discard = discard.into(); }
	}
	
	let sheet_params = calculate_sheet_params(demuxer.duration(), decoder.width(), decoder.height());
	
	let mut scaler_cache: Option<scaling::Context> = None;
	
	let mut sprite_sheet = RgbImage::new(
		sheet_params.sheet_cols * sheet_params.thumbnail_width,
		sheet_params.sheet_rows * sheet_params.thumbnail_height
	);
	
	// for offset in 0..sheet_params.thumbnail_count {
	// 	let time = (offset * sheet_params.interval).rescale(SEC_TIME_BASE, rescale::TIME_BASE);
	// 	
	// 	demuxer.seek(time, time..).context("Seeking")?;
	// 	
	// 	let frame = match extract_frame(&mut demuxer, &mut decoder, video_stream_index)? {
	// 		Some(f) => f,
	// 		None => continue,
	// 	};
	// 	
	// 	let rgb_frame = scale_frame_rgb(&mut scaler_cache, &frame, sheet_params.thumbnail_width, sheet_params.thumbnail_height)?;
	// 	let image_view = frame_image_sample_rgb(&rgb_frame);
	// 	
	// 	let x_pos = (offset % sheet_params.sheet_cols) * sheet_params.thumbnail_width;
	// 	let y_pos = (offset / sheet_params.sheet_rows) * sheet_params.thumbnail_height;
	// 	
	// 	sprite_sheet.copy_from(&image_view.as_view::<Rgb<u8>>().unwrap(), x_pos, y_pos).unwrap();
	// }
	
	// let mut frame = frame::Video::empty();
	// let mut frames_processed = 0;
	// let mut last_dts = -20;
	// 
	// let mut packets_processed = 0;
	// 
	// let mut receive_frames = |decoder: &mut decoder::Video| -> anyhow::Result<()> {
	// 	while decoder.receive_frame(&mut frame).is_ok() {
	// 		let rgb_frame = scale_frame_rgb(&mut scaler_cache, &frame, sheet_params.thumbnail_width, sheet_params.thumbnail_height)?;
	// 		let image_view = frame_image_sample_rgb(&rgb_frame);
	// 		
	// 		let current_count = frame.pts().unwrap().rescale(rescale::TIME_BASE, SEC_TIME_BASE) as u32 / sheet_params.interval;
	// 		
	// 		for _ in 0..(current_count.max(frames_processed) - frames_processed) {
	// 			let x_pos = (frames_processed % sheet_params.sheet_cols) * sheet_params.thumbnail_width;
	// 			let y_pos = (frames_processed / sheet_params.sheet_rows) * sheet_params.thumbnail_height;
	// 			
	// 			sprite_sheet.copy_from(&image_view.as_view::<Rgb<u8>>().unwrap(), x_pos, y_pos).unwrap();
	// 			frames_processed += 1;
	// 		}
	// 	}
	// 	
	// 	Ok(())
	// };
	// 
	// for result in demuxer.packets() {
	// 	let (stream, mut packet) = result?;
	// 	
	// 	if stream.index() == video_stream_index && packet.is_key() {
	// 		packet.rescale_ts(stream.time_base(), rescale::TIME_BASE);
	// 		let dts = packet.dts().unwrap();
	// 		
	// 		if dts >= (last_dts + sheet_params.interval.rescale(SEC_TIME_BASE, rescale::TIME_BASE)) {
	// 			// println!("Gap: {}", (dts - last_dts).rescale(rescale::TIME_BASE, SEC_TIME_BASE));
	// 			last_dts = dts;
	// 			
	// 			decoder.send_packet(&packet).context("Decoding packet")?;
	// 			receive_frames(&mut decoder)?;
	// 			packets_processed += 1;
	// 		}
	// 	}
	// }
	// 
	// decoder.send_eof()?;
	// receive_frames(&mut decoder)?;
	// 
	// println!("Frames processed: {}, expected: {}", frames_processed, sheet_params.thumbnail_count);
	// println!("Packets processed: {}", packets_processed);
	
	let mut frame = frame::Video::empty();
	let mut frames_processed = 0;
	
	let mut receive_frames = |decoder: &mut decoder::Video| -> anyhow::Result<()> {
		while decoder.receive_frame(&mut frame).is_ok() {
			let rgb_frame = scale_frame_rgb(&mut scaler_cache, &frame, sheet_params.thumbnail_width, sheet_params.thumbnail_height)?;
			let image_view = frame_image_sample_rgb(&rgb_frame);
			
			let x_pos = (frames_processed % sheet_params.sheet_cols) * sheet_params.thumbnail_width;
			let y_pos = (frames_processed / sheet_params.sheet_rows) * sheet_params.thumbnail_height;
			
			sprite_sheet.copy_from(&image_view.as_view::<Rgb<u8>>().unwrap(), x_pos, y_pos).unwrap();
			frames_processed += 1;
		}
		
		Ok(())
	};
	
	for offset in 0..sheet_params.thumbnail_count {
		let time = (offset * sheet_params.interval).rescale(SEC_TIME_BASE, rescale::TIME_BASE);
		
		demuxer.seek(time, time..).context("Seeking")?;
		
		for result in demuxer.packets() {
			let (stream, packet) = result?;
			
			if stream.index() == video_stream_index && packet.is_key() {
				decoder.send_packet(&packet).context("Decoding packet")?;
				receive_frames(&mut decoder)?;
				
				break;
			}
		}
	}
	
	decoder.send_eof()?;
	receive_frames(&mut decoder)?;
	
	let output_buffer = turbojpeg::compress_image(&sprite_sheet, JPEG_QUALITY, Subsamp::Sub2x2).unwrap();
	
	Ok((Bytes::from(output_buffer.to_vec()), sheet_params))
}