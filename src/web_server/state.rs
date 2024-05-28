use std::path::PathBuf;

use crate::config::ServerConfig;
use crate::services::thumbnail_service::ThumbnailService;
use crate::services::thumbnail_sheet_service::ThumbnailSheetService;
use crate::web_server::libraries::Libraries;
use crate::web_server::video_metadata::MediaMetadataCache;

pub struct ServerState {
	pub server_config: ServerConfig,
	pub libraries: Libraries,
	pub video_metadata_cache: MediaMetadataCache,
	pub thumbnail_generator: ThumbnailService,
	pub thumbnail_sheet_generator: ThumbnailSheetService,
}

impl ServerState {
	pub async fn init(server_config: ServerConfig) -> anyhow::Result<Self> {
		let libraries = Libraries::new(&server_config.libraries_config);
		
		Ok(Self {
			server_config,
			libraries,
			video_metadata_cache: MediaMetadataCache::new(),
			thumbnail_generator: ThumbnailService::init(PathBuf::from("cache/thumbnail")).await?, // TODO: Add config option for cache path
			thumbnail_sheet_generator: ThumbnailSheetService::init(PathBuf::from("cache/timeline-thumbnail")).await?,
		})
	}
}
