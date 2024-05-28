use std::path::PathBuf;

use anyhow::{anyhow, Context};
use bytes::Bytes;
use ffmpeg_the_third as ffmpeg;
use ffmpeg_the_third::{Rational, Rescale, rescale};
use ffmpeg_the_third::software::scaling;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use tokio::sync::Semaphore;
use turbojpeg::Subsamp;

use crate::services::artifact_cache::{ArtifactCache, CacheEntry, FileValidityKey};
use crate::transcoding::backends::software::SoftwareVideoBackend;
use crate::transcoding::backends::VideoBackend;
use crate::transcoding::image_utils::{extract_frame, scale_frame_rgb};

pub struct ThumbnailService {
	cache: ArtifactCache<FileValidityKey>,
	limiter: Semaphore,
}

impl ThumbnailService {
	pub async fn init(cache_dir: PathBuf) -> anyhow::Result<Self> {
		Ok(Self {
			cache: ArtifactCache::<FileValidityKey>::new(cache_dir).await?,
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

const MICRO_TIME_BASE: Rational = Rational(1, 1_000_000);

fn extract_thumbnail(media_path: PathBuf) -> anyhow::Result<(Bytes, ())> {
	let mut demuxer = ffmpeg::format::input(&media_path).context("Opening video file")?;
	
	let video_stream = demuxer.streams().best(ffmpeg::media::Type::Video).unwrap();
	let video_stream_index = video_stream.index();
	
	let mut video_backend = SoftwareVideoBackend::new();
	let mut decoder = video_backend.create_decoder(video_stream.parameters())?;
	
	let video_duration = demuxer.duration().rescale(rescale::TIME_BASE, MICRO_TIME_BASE);
	let duration_hash = blake3::hash(&video_duration.to_le_bytes());
	let mut rng = ChaCha20Rng::from_seed(duration_hash.as_bytes().to_owned());
	
	let mut scaler_cache: Option<scaling::Context> = None;
	let mut best_frame: Option<turbojpeg::OwnedBuf> = None;
	
	for _ in 0..5 {
		let time = rng.gen_range((video_duration / 10)..(video_duration / 10 * 9))
			.rescale(MICRO_TIME_BASE, rescale::TIME_BASE);
		
		demuxer.seek(time, time..).context("Seeking")?;
		
		let frame = match extract_frame(&mut demuxer, &mut decoder, video_stream_index)? {
			Some(f) => f,
			None => continue,
		};
		
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
	
	let output_buffer = best_frame.ok_or_else(|| anyhow!("No thumbnails found"))?;
	
	Ok((Bytes::from(output_buffer.to_vec()), ()))
}