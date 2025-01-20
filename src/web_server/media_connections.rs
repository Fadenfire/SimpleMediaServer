use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

use crate::web_server::libraries::Library;
use relative_path::{RelativePath, RelativePathBuf};
use serde::de::Error;
use serde::{Deserialize, Deserializer};

pub const CONNECTIONS_FILE_EXT: &str = "connections.json";

pub async fn get_video_connections(media_path: &Path, library_path: &RelativePath, library: &Library) -> anyhow::Result<Vec<ConnectedVideoEntry>> {
	let mut connections = Vec::new();
	
	let connections_path = media_path.with_extension(CONNECTIONS_FILE_EXT);
	
	if tokio::fs::try_exists(&connections_path).await? {
		let data = tokio::fs::read(&connections_path).await?;
		let connections_file: ConnectionsFile = serde_json::from_slice(&data)?;
		
		connections.extend(connections_file.connected_videos);
	}
	
	if let Some(global_connections_path) = &library.global_connections_file {
		let data = tokio::fs::read(global_connections_path).await?;
		let global_connections: GlobalConnectionsFile = serde_json::from_slice(&data)?;
		
		if let Some(conns) = global_connections.video_connections.get(library_path) {
			connections.extend_from_slice(conns);
		}
	}
	
	Ok(connections)
}

#[derive(Debug, Clone, Deserialize)]
pub struct GlobalConnectionsFile {
	pub video_connections: HashMap<RelativePathBuf, Vec<ConnectedVideoEntry>>,
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