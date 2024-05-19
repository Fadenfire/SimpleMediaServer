use std::path::PathBuf;

use crate::config::ServerConfig;
use crate::services::thumbnail_service::ThumbnailService;
use crate::web_server::libraries::Libraries;
use crate::web_server::video_metadata::VideoMetadataCache;

pub struct ServerState {
	pub server_config: ServerConfig,
	pub libraries: Libraries,
	pub video_metadata_cache: VideoMetadataCache,
	pub thumbnail_extractor: ThumbnailService,
}

impl ServerState {
	pub async fn init(server_config: ServerConfig) -> anyhow::Result<Self> {
		let libraries = Libraries::new(&server_config.libraries_config);
		
		Ok(Self {
			server_config,
			libraries,
			video_metadata_cache: VideoMetadataCache::new(),
			thumbnail_extractor: ThumbnailService::init(PathBuf::from("cache/thumbnail")).await?, // TODO: Add config option for cache path
		})
	}
}
