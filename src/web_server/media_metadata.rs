use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use anyhow::{anyhow, Context};
use ffmpeg_next::{codec, format, Rational, Rescale, rescale};
use ffmpeg_next::media::Type;
use serde::{Deserialize, Serialize};
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;
use time::{Date, OffsetDateTime};
use crate::media_manipulation::media_utils::MILLIS_TIME_BASE;
use crate::media_manipulation::thumbnail_sheet;
use crate::media_manipulation::thumbnail_sheet::ThumbnailSheetParams;
use crate::web_server::services::artifact_cache::ArtifactCache;
use crate::web_server::services::thumbnail_sheet_service::ThumbnailSheetGenerator;

pub struct MediaMetadataCache {
	metadata_cache: Mutex<HashMap<PathBuf, MetadataEntry>>,
}

struct MetadataEntry {
	basic: BasicMediaMetadata,
	advanced: Option<AdvancedMediaMetadata>,
}

impl MetadataEntry {
	fn still_valid(&self, file_metadata: &std::fs::Metadata) -> bool {
		self.basic.file_size == file_metadata.len() &&
			Some(self.basic.mod_time) == file_metadata.modified().ok()
	}
}

impl MediaMetadataCache {
	pub fn new() -> Self {
		Self {
			metadata_cache: Mutex::new(HashMap::new()),
		}
	}
	
	pub async fn fetch_basic_metadata(&self, media_path: impl AsRef<Path>) -> anyhow::Result<BasicMediaMetadata> {
		let media_path = media_path.as_ref().to_owned();
		let file_metadata = tokio::fs::metadata(&media_path).await?;
		
		{
			let cache = self.metadata_cache.lock().unwrap();
			
			if let Some(entry) = cache.get(&media_path) {
				if entry.still_valid(&file_metadata) {
					return Ok(entry.basic.clone());
				}
			}
		}
		
		let media_path2 = media_path.clone();
		
		let basic_metadata = tokio::task::spawn_blocking(move || {
			extract_basic_metadata(&media_path2, &file_metadata).map(|r| r.0)
		}).await.unwrap()?;
		
		{
			let mut cache = self.metadata_cache.lock().unwrap();
			
			cache.insert(media_path.to_owned(), MetadataEntry {
				basic: basic_metadata.clone(),
				advanced: None,
			});
		}
		
		Ok(basic_metadata)
	}
	
	pub async fn fetch_full_metadata(&self,
		media_path: impl AsRef<Path>,
		thumbnail_sheet_cache: &ArtifactCache<ThumbnailSheetGenerator>
	) -> anyhow::Result<(BasicMediaMetadata, AdvancedMediaMetadata)> {
		let media_path = media_path.as_ref().to_owned();
		let file_metadata = tokio::fs::metadata(&media_path).await?;
		
		{
			let cache = self.metadata_cache.lock().unwrap();
			
			if let Some(entry @ MetadataEntry {
				basic,
				advanced: Some(advanced)
			}) = cache.get(&media_path) {
				if entry.still_valid(&file_metadata) {
					return Ok((basic.clone(), advanced.clone()));
				}
			}
		}
		
		let thumbnail_sheet_params = thumbnail_sheet_cache.get(&media_path).await?.map(|entry| entry.metadata);
		let media_path2 = media_path.clone();
		
		let (basic_metadata, advanced_metadata) = tokio::task::spawn_blocking(move || {
			extract_full_metadata(&media_path2, &file_metadata, thumbnail_sheet_params)
		}).await.unwrap()?;
		
		{
			let mut cache = self.metadata_cache.lock().unwrap();
			
			cache.insert(media_path.to_owned(), MetadataEntry {
				basic: basic_metadata.clone(),
				advanced: Some(advanced_metadata.clone()),
			});
		}
		
		Ok((basic_metadata, advanced_metadata))
	}
}

#[derive(Clone, Debug)]
pub struct BasicMediaMetadata {
	pub file_size: u64,
	pub mod_time: SystemTime,
	pub path_name: String,
	pub duration: Duration,
	pub title: String,
	pub artist: Option<String>,
	pub creation_date: OffsetDateTime,
}

#[derive(Clone, Debug)]
pub struct AdvancedMediaMetadata {
	pub video_metadata: Option<VideoMetadata>,
}

#[derive(Clone, Debug)]
pub struct VideoMetadata {
	pub video_size: Dimension,
	pub thumbnail_sheet_params: ThumbnailSheetParams,
	pub frame_rate: Rational,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dimension {
	pub width: u32,
	pub height: u32,
}

const YT_DLP_DATE_FORMAT: &[BorrowedFormatItem] = format_description!("[year][month][day]");

fn extract_basic_metadata(
	media_path: &Path,
	file_metadata: &std::fs::Metadata,
) -> anyhow::Result<(BasicMediaMetadata, Option<format::context::Input>)> {
	let demuxer = format::input(&media_path).context("Opening video file")?;
	
	let mod_time = file_metadata.modified().context("FS doesn't support mod time")?;
	
	let duration_millis = demuxer.duration()
		.rescale(rescale::TIME_BASE, MILLIS_TIME_BASE)
		.try_into()
		.unwrap_or(0);
	let duration = Duration::from_millis(duration_millis);
	
	let path_name = media_path.file_stem()
		.and_then(OsStr::to_str)
		.ok_or_else(|| anyhow!("Path name is invalid"))?
		.to_owned();
	
	let title = demuxer.metadata().get("title")
		.map(ToOwned::to_owned)
		.unwrap_or_else(|| path_name.clone());
	
	let artist = demuxer.metadata().get("artist").map(ToOwned::to_owned);
	
	let creation_date = demuxer.metadata().get("date")
		.and_then(|date| Date::parse(date, YT_DLP_DATE_FORMAT).ok())
		.map(|date| date.midnight().assume_utc())
		.or_else(|| {
			file_metadata.created().ok()
				// If mod time is before ctime then something must have intentionally set it (probably trying to
				//  preserve the date). So use mod time instead of ctime as it's probably more accurate in that case.
				.filter(|c_time| c_time <= &mod_time)
				.map(Into::into)
		})
		.unwrap_or_else(|| mod_time.into());
	
	let ffmpeg_demuxer = Some(demuxer);
	
	let basic_metadata = BasicMediaMetadata {
		file_size: file_metadata.len(),
		mod_time,
		path_name,
		duration,
		title,
		artist,
		creation_date,
	};
	
	Ok((basic_metadata, ffmpeg_demuxer))
}

fn extract_full_metadata(
	media_path: &Path,
	file_metadata: &std::fs::Metadata,
	cached_thumbnail_sheet_params: Option<ThumbnailSheetParams>
) -> anyhow::Result<(BasicMediaMetadata, AdvancedMediaMetadata)> {
	let (basic_metadata, demuxer) = extract_basic_metadata(media_path, file_metadata)?;
	
	let demuxer = match demuxer {
		Some(demuxer) => demuxer,
		None => format::input(media_path).context("Opening video file")?,
	};
	
	let video_metadata = match demuxer.streams().best(Type::Video) {
		Some(video_stream) => {
			let decoder = codec::context::Context::from_parameters(video_stream.parameters())?
				.decoder().video().context("Opening decoder")?;
			
			let thumbnail_sheet_params = cached_thumbnail_sheet_params.unwrap_or_else(|| {
				thumbnail_sheet::calculate_sheet_params(demuxer.duration(), decoder.width(), decoder.height())
			});
			
			Some(VideoMetadata {
				video_size: Dimension {
					width: decoder.width(),
					height: decoder.height(),
				},
				thumbnail_sheet_params,
				frame_rate: decoder.frame_rate().unwrap_or(Rational(60, 1)),
			})
		}
		None => None
	};
	
	let advanced_metadata = AdvancedMediaMetadata {
		video_metadata,
	};
	
	Ok((basic_metadata, advanced_metadata))
}
