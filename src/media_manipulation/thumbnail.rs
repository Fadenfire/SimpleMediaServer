use std::path::PathBuf;

use anyhow::{anyhow, Context};
use bytes::Bytes;
use ffmpeg_next::{decoder, format, frame, media, rescale, Rescale};
use ffmpeg_next::software::scaling;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use turbojpeg::Subsamp;

use crate::media_manipulation::backends::software::SoftwareVideoBackend;
use crate::media_manipulation::backends::VideoBackend;
use crate::media_manipulation::media_utils::{discard_all_but_keyframes, MICRO_TIME_BASE, push_one_packet, scale_frame_rgb};

const TARGET_THUMBNAIL_HEIGHT: u32 = 720;
const JPEG_QUALITY: i32 = 90;

pub fn extract_thumbnail(media_path: PathBuf) -> anyhow::Result<Bytes> {
	let mut demuxer = format::input(&media_path).context("Opening video file")?;
	
	let video_stream = demuxer.streams().best(media::Type::Video).unwrap();
	let video_stream_index = video_stream.index();
	
	let mut video_backend = SoftwareVideoBackend::new();
	let mut decoder = video_backend.create_decoder(video_stream.parameters(), video_stream.time_base())?;
	
	discard_all_but_keyframes(&mut demuxer, video_stream_index);
	
	let video_duration = demuxer.duration().rescale(rescale::TIME_BASE, MICRO_TIME_BASE);
	let duration_hash = blake3::hash(&video_duration.to_le_bytes());
	let mut rng = ChaCha20Rng::from_seed(duration_hash.as_bytes().to_owned());
	
	let mut scaler_cache: Option<scaling::Context> = None;
	let mut best_frame: Option<turbojpeg::OwnedBuf> = None;
	
	let mut frame = frame::Video::empty();
	
	let mut receive_frames = |decoder: &mut decoder::Video| -> anyhow::Result<()> {
		while decoder.receive_frame(&mut frame).is_ok() {
			let out_height = frame.height().min(TARGET_THUMBNAIL_HEIGHT);
			let out_width = frame.width() * out_height / frame.height();
			
			let rgb_frame = scale_frame_rgb(&mut scaler_cache, &frame, out_width, out_height)?;
			
			let image = turbojpeg::Image {
				pixels: rgb_frame.data(0),
				width: rgb_frame.width() as usize,
				pitch: rgb_frame.stride(0),
				height: rgb_frame.height() as usize,
				format: turbojpeg::PixelFormat::RGB,
			};
			
			let output_buffer = turbojpeg::compress(image, JPEG_QUALITY, Subsamp::Sub2x2).unwrap();
			
			// The frame with the largest size after JPEG compression should have the highest entropy and be
			// the most interesting looking thumbnail
			if best_frame.is_none() || output_buffer.len() > best_frame.as_ref().unwrap().len() {
				best_frame = Some(output_buffer);
			}
		}
		
		Ok(())
	};
	
	for _ in 0..5 {
		let time = rng.gen_range((video_duration / 10)..(video_duration / 10 * 9))
			.rescale(MICRO_TIME_BASE, rescale::TIME_BASE);
		
		demuxer.seek(time, time..).context("Seeking")?;
		
		push_one_packet(&mut demuxer, &mut decoder, video_stream_index)?;
		receive_frames(&mut decoder)?;
	}
	
	decoder.send_eof()?;
	receive_frames(&mut decoder)?;
	
	let output_buffer = best_frame.ok_or_else(|| anyhow!("No thumbnails found"))?;
	
	Ok(Bytes::from(output_buffer.to_vec()))
}