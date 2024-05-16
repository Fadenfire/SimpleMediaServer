use crate::config::ServerConfig;
use crate::web_server::libraries::Libraries;
use crate::web_server::video_metadata::VideoMetadataCache;

pub struct ServerState {
	pub server_config: ServerConfig,
	pub libraries: Libraries,
	pub video_metadata_cache: VideoMetadataCache,
}

impl ServerState {
	pub fn new(server_config: ServerConfig) -> Self {
		let libraries = Libraries::new(&server_config.libraries_config);
		
		Self {
			server_config,
			libraries,
			video_metadata_cache: VideoMetadataCache::new(),
		}
	}
}
