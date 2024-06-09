use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use anyhow::Context;
use ffmpeg_next::{codec, format, Rescale, rescale};
use ffmpeg_next::media::Type;
use serde::{Deserialize, Serialize};

use crate::media_manipulation::media_utils::MILLIS_TIME_BASE;
use crate::media_manipulation::thumbnail_sheet;
use crate::media_manipulation::thumbnail_sheet::ThumbnailSheetParams;
use crate::services::artifact_cache::ArtifactCache;
use crate::services::thumbnail_sheet_service::ThumbnailSheetGenerator;

pub struct MediaMetadataCache {
	metadata_cache: Mutex<HashMap<PathBuf, MediaMetadata>>,
}

impl MediaMetadataCache {
	pub fn new() -> Self {
		Self {
			metadata_cache: Mutex::new(HashMap::new()),
		}
	}
	
	pub async fn fetch_media_metadata(&self, media_path: impl AsRef<Path>, thumbnail_sheet_cache: &ArtifactCache<ThumbnailSheetGenerator>) -> anyhow::Result<MediaMetadata> {
		let media_path = media_path.as_ref().to_owned();
		let file_metadata = tokio::fs::metadata(&media_path).await?;
		
		{
			let cache = self.metadata_cache.lock().unwrap();
			
			if let Some(media_metadata) = cache.get(&media_path) {
				if media_metadata.file_size == file_metadata.len() && media_metadata.mod_time == file_metadata.modified().ok() {
					return Ok(media_metadata.clone());
				}
			}
		}
		
		let thumbnail_sheet_params = thumbnail_sheet_cache.get(&media_path).await?.map(|entry| entry.metadata);
		let video_path2 = media_path.clone();
		
		let media_metadata = tokio::task::spawn_blocking(move || -> anyhow::Result<MediaMetadata> {
			let demuxer = format::input(&video_path2).context("Opening video file")?;
			
			let duration_millis = demuxer.duration()
				.rescale(rescale::TIME_BASE, MILLIS_TIME_BASE)
				.try_into()
				.unwrap_or(0);
			let duration = Duration::from_millis(duration_millis);
			
			let title = demuxer.metadata().get("title")
				.map(ToOwned::to_owned)
				.or_else(|| video_path2.file_stem().map(OsStr::to_string_lossy).map(Cow::into_owned))
				.unwrap_or_else(|| "Unknown".to_owned());
			
			let artist = demuxer.metadata().get("artist").map(ToOwned::to_owned);
			
			let video_metadata = match demuxer.streams().best(Type::Video) {
				Some(video_stream) => {
					let decoder = codec::context::Context::from_parameters(video_stream.parameters())?
						.decoder().video().context("Opening decoder")?;
					
					let thumbnail_sheet_params = thumbnail_sheet_params.unwrap_or_else(|| {
						thumbnail_sheet::calculate_sheet_params(demuxer.duration(), decoder.width(), decoder.height())
					});
					
					Some(VideoMetadata {
						video_size: Dimension {
							width: decoder.width(),
							height: decoder.height(),
						},
						thumbnail_sheet_params,
					})
				}
				None => None
			};
			
			Ok(MediaMetadata {
				file_path: video_path2,
				file_size: file_metadata.len(),
				mod_time: file_metadata.modified().ok(),
				duration,
				title,
				artist,
				video_metadata,
			})
		}).await.unwrap()?;
		
		{
			let mut cache = self.metadata_cache.lock().unwrap();
			cache.insert(media_path.to_owned(), media_metadata.clone());
		}
		
		Ok(media_metadata)
	}
}

#[derive(Clone, Debug)]
pub struct MediaMetadata {
	pub file_path: PathBuf,
	pub file_size: u64,
	pub mod_time: Option<SystemTime>,
	pub duration: Duration,
	pub title: String,
	pub artist: Option<String>,
	pub video_metadata: Option<VideoMetadata>,
}

#[derive(Clone, Debug)]
pub struct VideoMetadata {
	pub video_size: Dimension,
	pub thumbnail_sheet_params: ThumbnailSheetParams,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dimension {
	pub width: u32,
	pub height: u32,
}