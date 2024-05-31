use std::path::PathBuf;
use std::sync::Arc;

use http_body_util::BodyExt;
use tower_http::services::{ServeDir, ServeFile};
use tower_service::Service;
use tracing::instrument;

use crate::config::ServerConfig;
use crate::services::{hls_segment_service, thumbnail_service, thumbnail_sheet_service};
use crate::services::artifact_cache::ArtifactCache;
use crate::services::hls_segment_service::HlsSegmentGenerator;
use crate::services::thumbnail_service::ThumbnailGenerator;
use crate::services::thumbnail_sheet_service::ThumbnailSheetGenerator;
use crate::web_server::api_routes;
use crate::web_server::libraries::Libraries;
use crate::web_server::video_metadata::MediaMetadataCache;
use crate::web_server::web_utils::{HyperRequest, HyperResponse};

pub struct ServerState {
	pub server_config: ServerConfig,
	
	serve_web_ui: ServeDir<ServeFile>,
	
	pub libraries: Libraries,
	pub video_metadata_cache: MediaMetadataCache,
	pub thumbnail_generator: ArtifactCache<ThumbnailGenerator>,
	pub thumbnail_sheet_generator: ArtifactCache<ThumbnailSheetGenerator>,
	pub hls_segment_generator: ArtifactCache<HlsSegmentGenerator>,
}

impl ServerState {
	pub async fn init(server_config: ServerConfig, web_ui_dir: PathBuf) -> anyhow::Result<Self> {
		let libraries = Libraries::new(&server_config.libraries_config);
		
		let web_ui_index = ServeFile::new(web_ui_dir.join("index.html"))
			.precompressed_gzip()
			.precompressed_br();
		
		let serve_web_ui = ServeDir::new(&web_ui_dir)
			.precompressed_gzip()
			.precompressed_br()
			.fallback(web_ui_index);
		
		Ok(Self {
			server_config,
			
			serve_web_ui,
			
			libraries,
			video_metadata_cache: MediaMetadataCache::new(),
			thumbnail_generator: thumbnail_service::init_service(PathBuf::from("cache/thumbnail")).await?, // TODO: Add config option for cache path
			thumbnail_sheet_generator: thumbnail_sheet_service::init_service(PathBuf::from("cache/timeline-thumbnail")).await?,
			hls_segment_generator: hls_segment_service::init_service(PathBuf::from("cache/segments")).await?,
		})
	}
}

#[instrument(skip(request, server_state))]
pub async fn route_request(request: HyperRequest, path: &[&str], server_state: Arc<ServerState>) -> HyperResponse {
	match path {
		["api", tail @ ..] => api_routes::route_request(request, tail, server_state).await,
		
		_ => {
			server_state.serve_web_ui.clone().call(request).await
				.unwrap()
				.map(|body| body.map_err(anyhow::Error::new).boxed_unsync())
		}
	}
}