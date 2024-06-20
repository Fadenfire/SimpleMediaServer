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
use crate::web_server::services::artifact_cache::ArtifactCache;
use crate::web_server::services::hls_segment_service::HlsSegmentGenerator;
use crate::web_server::services::task_pool::TaskPool;
use crate::web_server::services::thumbnail_service::ThumbnailGenerator;
use crate::web_server::services::thumbnail_sheet_service::ThumbnailSheetGenerator;
use crate::web_server::video_metadata::MediaMetadataCache;
use crate::web_server::web_utils::{HyperRequest, HyperResponse};

pub struct ServerState {
	pub server_config: ServerConfig,
	
	serve_web_ui: ServeDir<ServeFile>,
	
	pub libraries: Libraries,
	pub video_metadata_cache: MediaMetadataCache,
	
	pub media_backend_factory: Arc<MediaBackendFactory>,
	pub transcoding_task_pool: Arc<TaskPool>,
	
	pub hls_segment_generator: ArtifactCache<HlsSegmentGenerator>,
	pub thumbnail_generator: ArtifactCache<ThumbnailGenerator>,
	pub thumbnail_sheet_generator: ArtifactCache<ThumbnailSheetGenerator>,
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
		
		let media_backend_factory = Arc::new(MediaBackendFactory::new(server_config.main_config.transcoding.backend)?);
		let transcoding_task_pool = Arc::new(TaskPool::new(server_config.main_config.transcoding.concurrent_tasks));
		
		let hls_segment_generator = ArtifactCache::builder()
			.cache_dir(server_config.paths.transcoded_segments_cache_dir.clone())
			.task_pool(transcoding_task_pool.clone())
			.file_size_limit(server_config.main_config.caches.segments_cache_size_limit)
			.build(HlsSegmentGenerator::new(media_backend_factory.clone()))
			.await?;
		
		info!("HLS segments cache contains {} bytes, {} bytes max", hls_segment_generator.cache_size(),
			server_config.main_config.caches.segments_cache_size_limit);
		
		let thumbnail_generator = ArtifactCache::builder()
			.cache_dir(server_config.paths.thumbnail_cache_dir.clone())
			.task_pool(transcoding_task_pool.clone())
			.file_size_limit(server_config.main_config.caches.thumbnail_cache_size_limit)
			.build(ThumbnailGenerator::new(media_backend_factory.clone()))
			.await?;
		
		info!("Thumbnail cache contains {} bytes, {} bytes max", thumbnail_generator.cache_size(),
			server_config.main_config.caches.thumbnail_cache_size_limit);
		
		let thumbnail_sheet_generator = ArtifactCache::builder()
			.cache_dir(server_config.paths.thumbnail_sheet_cache_dir.clone())
			.task_pool(transcoding_task_pool.clone())
			.file_size_limit(server_config.main_config.caches.thumbnail_sheet_cache_size_limit)
			.build(ThumbnailSheetGenerator::new(media_backend_factory.clone()))
			.await?;
		
		info!("Thumbnail sheet cache contains {} bytes, {} bytes max", thumbnail_sheet_generator.cache_size(),
			server_config.main_config.caches.thumbnail_sheet_cache_size_limit);
		
		let video_metadata_cache = MediaMetadataCache::new();
		
		Ok(Self {
			server_config,
			
			serve_web_ui,
			
			libraries,
			video_metadata_cache,
			
			media_backend_factory,
			transcoding_task_pool,
			
			hls_segment_generator,
			thumbnail_generator,
			thumbnail_sheet_generator,
		})
	}
}

#[instrument(skip_all)]
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