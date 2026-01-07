use std::ffi::OsStr;
use std::path::Path;
use std::time::Duration;

use crate::media_manipulation::media_utils::MILLIS_TIME_BASE;
use crate::web_server::metadata_cache::FileMetadata;
use crate::web_server::video_locator::{MKV_EXTENSIONS, MP4_EXTENSIONS};
use anyhow::{anyhow, Context};
use ffmpeg_next::media::Type;
use ffmpeg_next::{codec, format, rescale, Rational, Rescale};
use matroska::TagValue;
use serde::{Deserialize, Serialize};
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;
use time::{Date, OffsetDateTime};

#[derive(Clone, Debug)]
pub struct BasicMediaMetadata {
	pub file_size: u64,
	pub path_name: String,
	pub duration: Duration,
	pub title: String,
	pub artist: Option<String>,
	pub creation_date: OffsetDateTime,
}

impl FileMetadata for BasicMediaMetadata {
	async fn fetch_metadata(media_path: &Path, file_metadata: &std::fs::Metadata) -> anyhow::Result<Self> {
		let media_path = media_path.to_owned();
		let file_metadata = file_metadata.clone();
		
		tokio::task::spawn_blocking(move || extract_basic_metadata(&media_path, &file_metadata)).await?
	}
}

#[derive(Clone, Debug)]
pub struct AdvancedMediaMetadata {
	pub ffmpeg_duration: Duration,
	pub video_metadata: Option<VideoMetadata>,
	pub subtitle_streams: Vec<SubtitleStream>,
}

#[derive(Clone, Debug)]
pub struct VideoMetadata {
	pub video_size: Dimension,
	pub frame_rate: Rational,
}

#[derive(Clone, Debug)]
pub struct SubtitleStream {
	pub index: usize,
	pub language: Option<String>,
	pub name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dimension {
	pub width: u32,
	pub height: u32,
}

impl FileMetadata for AdvancedMediaMetadata {
	async fn fetch_metadata(media_path: &Path, _file_metadata: &std::fs::Metadata) -> anyhow::Result<Self> {
		let media_path = media_path.to_owned();
		
		tokio::task::spawn_blocking(move || extract_advanced_metadata(&media_path)).await?
	}
}

const YT_DLP_DATE_FORMAT: &[BorrowedFormatItem] = format_description!("[year][month][day]");

fn extract_basic_metadata(
	media_path: &Path,
	file_metadata: &std::fs::Metadata,
) -> anyhow::Result<BasicMediaMetadata> {
	let mod_time = file_metadata.modified().context("FS doesn't support mod time")?;
	
	let path_name = media_path.file_stem()
		.and_then(OsStr::to_str)
		.ok_or_else(|| anyhow!("Path name is invalid"))?
		.to_owned();
	
	let extension = media_path.extension().and_then(OsStr::to_str);
	
	let duration;
	let title;
	let mut artist;
	let mut creation_date;
	
	if extension.is_some_and(|ext| MP4_EXTENSIONS.contains(&ext)) {
		let mut read_config = mp4ameta::ReadConfig::NONE;
		read_config.read_meta_items = true;
		
		let tag = mp4ameta::Tag::read_with_path(media_path, &read_config)
			.context("Reading mp4 metadata")?;
		
		duration = tag.duration();
		
		title = tag.title().map(ToOwned::to_owned);
		artist = tag.artist().map(ToOwned::to_owned);
		creation_date = tag.year().map(ToOwned::to_owned);
	} else if extension.is_some_and(|ext| MKV_EXTENSIONS.contains(&ext)) {
		let mkv = matroska::open(media_path).context("Reading mkv metadata")?;
		
		duration = mkv.info.duration.ok_or_else(|| anyhow!("MKV is missing duration"))?;
		title = mkv.info.title;
		
		artist = None;
		creation_date = None;
		
		fn convert_tag_value(value: Option<TagValue>) -> Option<String> {
			match value {
				Some(TagValue::String(s)) => Some(s),
				_ => None,
			}
		}
		
		for tag in mkv.tags {
			for simple_tag in tag.simple {
				match simple_tag.name.as_str() {
					"ARTIST" => artist = convert_tag_value(simple_tag.value),
					"DATE" => creation_date = convert_tag_value(simple_tag.value),
					_ => {}
				}
			}
		}
	} else {
		let demuxer = format::input(media_path).context("Opening video file")?;
		
		let duration_millis = demuxer.duration()
			.rescale(rescale::TIME_BASE, MILLIS_TIME_BASE)
			.try_into()
			.unwrap_or(0);
		
		duration = Duration::from_millis(duration_millis);
		
		title = demuxer.metadata().get("title").map(ToOwned::to_owned);
		artist = demuxer.metadata().get("artist").map(ToOwned::to_owned);
		creation_date = demuxer.metadata().get("date").map(ToOwned::to_owned);
	}
	
	let title = title.unwrap_or_else(|| path_name.clone());
	
	let creation_date = creation_date
		.and_then(|date| Date::parse(&date, YT_DLP_DATE_FORMAT).ok())
		.map(|date| date.midnight().assume_utc())
		.or_else(|| {
			file_metadata.created().ok()
				// If mod time is before ctime then something must have intentionally set it (probably trying to
				//  preserve the date). So use mod time instead of ctime as it's probably more accurate in that case.
				.filter(|c_time| c_time <= &mod_time)
				.map(Into::into)
		})
		.unwrap_or_else(|| mod_time.into());
	
	Ok(BasicMediaMetadata {
		file_size: file_metadata.len(),
		path_name,
		duration,
		title,
		artist,
		creation_date,
	})
}

fn extract_advanced_metadata(media_path: &Path) -> anyhow::Result<AdvancedMediaMetadata> {
	let demuxer = format::input(media_path).context("Opening video file")?;
	
	let video_metadata = match demuxer.streams().best(Type::Video) {
		Some(video_stream) => {
			let decoder = codec::context::Context::from_parameters(video_stream.parameters())?
				.decoder().video().context("Opening decoder")?;
			
			Some(VideoMetadata {
				video_size: Dimension {
					width: decoder.width(),
					height: decoder.height(),
				},
				frame_rate: decoder.frame_rate().unwrap_or(Rational(60, 1)),
			})
		}
		None => None
	};
	
	let duration_millis = demuxer.duration()
		.rescale(rescale::TIME_BASE, MILLIS_TIME_BASE)
		.try_into()
		.unwrap_or(0);
	
	let ffmpeg_duration = Duration::from_millis(duration_millis);
	
	let subtitle_streams: Vec<_> = demuxer.streams()
		.filter(|stream| stream.parameters().medium() == Type::Subtitle)
		.map(|stream| {
			let language = stream.metadata().get("language").map(ToOwned::to_owned);
			
			let name = stream.metadata().get("handler_name")
				.filter(|name| *name != "SubtitleHandler")
				.map(ToOwned::to_owned);
			
			SubtitleStream {
				index: stream.index(),
				language,
				name,
			}
		})
		.collect();
	
	Ok(AdvancedMediaMetadata {
		ffmpeg_duration,
		video_metadata,
		subtitle_streams,
	})
}
