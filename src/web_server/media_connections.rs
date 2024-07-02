use std::path::Path;
use std::str::FromStr;

use relative_path::RelativePathBuf;
use serde::{Deserialize, Deserializer};
use serde::de::Error;

use crate::utils;

pub const CONNECTIONS_FILE_EXT: &str = "connections.json";

pub async fn get_video_connections(path: &Path) -> anyhow::Result<Option<ConnectionsFile>> {
	let connections_path = utils::add_extension(path, CONNECTIONS_FILE_EXT);
	
	if !tokio::fs::try_exists(&connections_path).await? {
		return Ok(None);
	}
	
	let data = tokio::fs::read(&connections_path).await?;
	Ok(Some(serde_json::from_slice(&data)?))
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectionsFile {
	pub connected_videos: Vec<ConnectedVideoEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectedVideoEntry {
	pub video_path: RelativePathBuf,
	pub relation: String,
	pub shortcut_thumbnail: Option<RelativePathBuf>,
	pub connections: Vec<ConnectionEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectionEntry {
	#[serde(deserialize_with = "deserialize_timestamp")]
	pub left_start: u64,
	#[serde(deserialize_with = "deserialize_timestamp")]
	pub right_start: u64,
	#[serde(deserialize_with = "deserialize_timestamp")]
	pub duration: u64,
}

fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<u64, D::Error>
where D: Deserializer<'de>
{
	let string = String::deserialize(deserializer)?;
	let mut split = string.split(':');
	
	let hours = split.next().and_then(|s| u64::from_str(s).ok()).ok_or_else(|| Error::custom("Invalid timestamp"))?;
	let minutes = split.next().and_then(|s| u64::from_str(s).ok()).ok_or_else(|| Error::custom("Invalid timestamp"))?;
	let seconds = split.next().and_then(|s| u64::from_str(s).ok()).ok_or_else(|| Error::custom("Invalid timestamp"))?;
	
	Ok(hours * 60 * 60 + minutes * 60 + seconds)
}