use std::path::PathBuf;

use anyhow::{anyhow, Context};
use bytes::Bytes;
use ffmpeg_the_third as ffmpeg;
use ffmpeg_the_third::{frame, Rational, Rescale, rescale};
use ffmpeg_the_third::format::Pixel;
use ffmpeg_the_third::software::scaling;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use tokio::sync::Semaphore;
use turbojpeg::Subsamp;

use crate::services::artifact_cache::{ArtifactCache, CacheEntry, FileValidityKey};
use crate::transcoding::backends::software::SoftwareVideoBackend;
use crate::transcoding::backends::VideoBackend;

pub struct ThumbnailService {
	cache: ArtifactCache<FileValidityKey>,
	limiter: Semaphore,
}

impl ThumbnailService {
	pub async fn init(cache_dir: PathBuf) -> anyhow::Result<Self> {
		Ok(Self {
			cache: ArtifactCache::new(cache_dir).await?,
			limiter: Semaphore::new(4),
		})
	}
	
	pub async fn extract_thumbnail(&self, media_path: PathBuf) -> anyhow::Result<CacheEntry> {
		let cache_key = format!("{}.jpg", blake3::hash(media_path.as_os_str().as_encoded_bytes()).to_hex());
		let validity_key = FileValidityKey::from_file(&media_path).await?;
		
		self.cache.get_or_insert(&cache_key, validity_key, || async {
			let _permit = self.limiter.acquire().await.unwrap();
			
			tokio::task::spawn_blocking(|| extract_thumbnail(media_path)).await.unwrap()
		}).await
	}
}

const TARGET_THUMBNAIL_HEIGHT: u32 = 720;
const JPEG_QUALITY: i32 = 90;

fn extract_thumbnail(media_path: PathBuf) -> anyhow::Result<Bytes> {
	let mut demuxer = ffmpeg::format::input(&media_path).context("Opening video file")?;
	
	let video_stream = demuxer.streams().best(ffmpeg::media::Type::Video).unwrap();
	let video_stream_index = video_stream.index();
	
	let mut video_backend = SoftwareVideoBackend::new();
	let mut decoder = video_backend.create_decoder(video_stream.parameters())?;
	
	const MICRO_TIME_BASE: Rational = Rational(1, 1_000_000);
	
	let video_duration = demuxer.duration().rescale(rescale::TIME_BASE, MICRO_TIME_BASE);
	let duration_hash = blake3::hash(&video_duration.to_le_bytes());
	let mut rng = ChaCha20Rng::from_seed(duration_hash.as_bytes().to_owned());
	
	let mut best_frame: Option<turbojpeg::OwnedBuf> = None;
	
	for _ in 0..5 {
		let time = rng.gen_range((video_duration / 10)..(video_duration / 10 * 9))
			.rescale(MICRO_TIME_BASE, rescale::TIME_BASE);
		
		demuxer.seek(time, time..).context("Seeking")?;
		
		let mut frame = frame::Video::empty();
		let mut found_frame = false;
		
		for result in demuxer.packets() {
			let (stream, packet) = result?;
			
			if stream.index() == video_stream_index {
				decoder.send_packet(&packet).context("Decoding packet")?;
				
				if decoder.receive_frame(&mut frame).is_ok() {
					found_frame = true;
					break;
				}
			}
		}
		
		if !found_frame {
			continue;
		}
		
		let out_height = frame.height().min(TARGET_THUMBNAIL_HEIGHT);
		let out_width = frame.width() * out_height / frame.height();
		
		let mut converter = scaling::Context::get(
			frame.format(), frame.width(), frame.height(),
			Pixel::RGB24, out_width, out_height,
			scaling::Flags::BICUBIC,
		).context("Creating image converter")?;
		
		let mut rgb_frame = frame::Video::empty();
		converter.run(&frame, &mut rgb_frame).context("Converting frame")?;
		
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
	
	let output_buffer = best_frame.ok_or_else(|| anyhow!("No thumbnails found"))?;
	
	Ok(Bytes::from(output_buffer.to_vec()))
}