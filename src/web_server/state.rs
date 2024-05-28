use std::path::PathBuf;

use crate::config::ServerConfig;
use crate::services::{thumbnail_service, thumbnail_sheet_service};
use crate::services::artifact_cache::ArtifactCache;
use crate::services::thumbnail_service::ThumbnailGenerator;
use crate::services::thumbnail_sheet_service::ThumbnailSheetGenerator;
use crate::web_server::libraries::Libraries;
use crate::web_server::video_metadata::MediaMetadataCache;

pub struct ServerState {
	pub server_config: ServerConfig,
	pub libraries: Libraries,
	pub video_metadata_cache: MediaMetadataCache,
	pub thumbnail_generator: ArtifactCache<ThumbnailGenerator>,
	pub thumbnail_sheet_generator: ArtifactCache<ThumbnailSheetGenerator>,
}

impl ServerState {
	pub async fn init(server_config: ServerConfig) -> anyhow::Result<Self> {
		let libraries = Libraries::new(&server_config.libraries_config);
		
		Ok(Self {
			server_config,
			libraries,
			video_metadata_cache: MediaMetadataCache::new(),
			thumbnail_generator: thumbnail_service::init_service(PathBuf::from("cache/thumbnail")).await?, // TODO: Add config option for cache path
			thumbnail_sheet_generator: thumbnail_sheet_service::init_service(PathBuf::from("cache/timeline-thumbnail")).await?,
		})
	}
}
