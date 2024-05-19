use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use anyhow::Context;
use ffmpeg_the_third as ffmpeg;
use ffmpeg_the_third::{codec, Rational, Rescale, rescale};
use ffmpeg_the_third::media::Type;
use serde::{Deserialize, Serialize};

pub struct VideoMetadataCache {
	metadata_cache: Mutex<HashMap<PathBuf, VideoMetadata>>,
}

impl VideoMetadataCache {
	pub fn new() -> Self {
		Self {
			metadata_cache: Mutex::new(HashMap::new()),
		}
	}
	
	pub async fn fetch_video_metadata(&self, video_path: impl AsRef<Path>) -> anyhow::Result<VideoMetadata> {
		let video_path = video_path.as_ref();
		let file_metadata = tokio::fs::metadata(video_path).await?;
		
		{
			let cache = self.metadata_cache.lock().unwrap();
			
			if let Some(video_metadata) = cache.get(video_path) {
				if video_metadata.size == file_metadata.len() && video_metadata.mod_time == file_metadata.modified().ok() {
					return Ok(video_metadata.clone());
				}
			}
		}
		
		let video_path2 = video_path.to_owned();
		
		let video_metadata = tokio::task::spawn_blocking(move || -> anyhow::Result<VideoMetadata> {
			let demuxer = ffmpeg::format::input(&video_path2).context("Opening video file")?;
			
			let duration_millis = demuxer.duration()
				.rescale(rescale::TIME_BASE, Rational(1, 1000))
				.try_into()
				.unwrap_or(0);
			let duration = Duration::from_millis(duration_millis);
			
			let title = demuxer.metadata().get("title")
				.map(ToOwned::to_owned)
				.or_else(|| video_path2.file_stem().map(OsStr::to_string_lossy).map(Cow::into_owned))
				.unwrap_or_else(|| "Unknown".to_owned());
			
			let artist = demuxer.metadata().get("artist").map(ToOwned::to_owned);
			
			let mut video_resolution = None;
			
			if let Some(video_stream) = demuxer.streams().best(Type::Video) {
				let decoder = codec::context::Context::from_parameters(video_stream.parameters())?
					.decoder().video().context("Opening decoder")?;
				
				video_resolution = Some(Dimension {
					width: decoder.width(),
					height: decoder.height(),
				})
			}
			
			Ok(VideoMetadata {
				file_path: video_path2,
				size: file_metadata.len(),
				mod_time: file_metadata.modified().ok(),
				duration,
				title,
				artist,
				video_resolution,
			})
		}).await.unwrap()?;
		
		{
			let mut cache = self.metadata_cache.lock().unwrap();
			cache.insert(video_path.to_owned(), video_metadata.clone());
		}
		
		Ok(video_metadata)
	}
}

#[derive(Clone, Debug)]
pub struct VideoMetadata {
	pub file_path: PathBuf,
	pub size: u64,
	pub mod_time: Option<SystemTime>,
	pub duration: Duration,
	pub title: String,
	pub artist: Option<String>,
	pub video_resolution: Option<Dimension>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dimension {
	pub width: u32,
	pub height: u32,
}