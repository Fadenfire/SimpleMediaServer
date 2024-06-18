use std::path::PathBuf;
use std::sync::Arc;

use http_body_util::BodyExt;
use tower_http::services::{ServeDir, ServeFile};
use tower_service::Service;
use tracing::{info, instrument};

use crate::config::ServerConfig;
use crate::web_server::api_routes;
use crate::web_server::libraries::Libraries;
use crate::web_server::media_backend_factory::MediaBackendFactory;
use crate::web_server::services::{hls_segment_service, thumbnail_service, thumbnail_sheet_service};
use crate::web_server::services::artifact_cache::ArtifactCache;
use crate::web_server::services::hls_segment_service::HlsSegmentGenerator;
use crate::web_server::services::thumbnail_service::ThumbnailGenerator;
use crate::web_server::services::thumbnail_sheet_service::ThumbnailSheetGenerator;
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
	pub media_backend_factory: Arc<MediaBackendFactory>,
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
		
		let media_backend_factory = Arc::new(MediaBackendFactory::new());
		
		Ok(Self {
			server_config,
			
			serve_web_ui,
			
			libraries,
			video_metadata_cache: MediaMetadataCache::new(),
			thumbnail_generator: thumbnail_service::init_service(PathBuf::from("cache/thumbnail"), media_backend_factory.clone()).await?, // TODO: Add config option for cache path
			thumbnail_sheet_generator: thumbnail_sheet_service::init_service(PathBuf::from("cache/timeline-thumbnail"), media_backend_factory.clone()).await?,
			hls_segment_generator: hls_segment_service::init_service(PathBuf::from("/tmp/media-server-segments-cache"), media_backend_factory.clone()).await?,
			media_backend_factory,
		})
	}
}

#[instrument(skip(request, server_state))]
pub async fn route_request(request: HyperRequest, path: &[&str], server_state: Arc<ServerState>) -> HyperResponse {
	info!("Request for {}", request.uri().path());
	
	match path {
		["api", tail @ ..] => api_routes::route_request(request, tail, server_state).await,
		
		_ => {
			server_state.serve_web_ui.clone().call(request).await
				.unwrap()
				.map(|body| body.map_err(anyhow::Error::new).boxed_unsync())
		}
	}
}