use std::sync::{Arc, Mutex};

use tracing::info;

use crate::config::ServerConfig;
use crate::utils;
use crate::web_server::auth::AuthManager;
use crate::web_server::libraries::Libraries;
use crate::web_server::media_backend_factory::MediaBackendFactory;
use crate::web_server::services::artifact_cache::ArtifactCache;
use crate::web_server::services::hls_segment_service::HlsSegmentGenerator;
use crate::web_server::services::task_pool::TaskPool;
use crate::web_server::services::thumbnail_service::ThumbnailGenerator;
use crate::web_server::services::thumbnail_sheet_service::ThumbnailSheetGenerator;
use crate::web_server::media_metadata::MediaMetadataCache;
use crate::web_server::watch_history::UserWatchHistories;

pub struct ServerState {
	pub config: ServerConfig,
	
	pub libraries: Libraries,
	pub auth_manager: AuthManager,
	pub user_watch_histories: Arc<Mutex<UserWatchHistories>>,
	pub video_metadata_cache: MediaMetadataCache,
	
	pub hls_segment_generator: ArtifactCache<HlsSegmentGenerator>,
	pub thumbnail_generator: ArtifactCache<ThumbnailGenerator>,
	pub thumbnail_sheet_generator: ArtifactCache<ThumbnailSheetGenerator>,
}

impl ServerState {
	pub async fn init(config: ServerConfig) -> anyhow::Result<Self> {
		let libraries = Libraries::from_config(config.load_libraries_config().await?);
		let auth_manager = AuthManager::from_config(config.load_users_config().await?);
		
		let user_watch_histories = UserWatchHistories::load(&auth_manager,
			config.paths.data_dir.join("watch-histories")).await?;
		
		let media_backend_factory = Arc::new(MediaBackendFactory::new(config.main_config.transcoding.backend)?);
		let transcoding_task_pool = Arc::new(TaskPool::new(config.main_config.transcoding.concurrent_tasks));
		
		let hls_segment_generator = ArtifactCache::builder()
			.cache_dir(config.paths.transcoded_segments_cache_dir.clone())
			.task_pool(transcoding_task_pool.clone())
			.file_size_limit(config.main_config.caches.segments_cache_size_limit)
			.build(HlsSegmentGenerator::new(media_backend_factory.clone()))
			.await?;
		
		info!("HLS segments cache contains {}B, {}B max",
			utils::abbreviate_number(hls_segment_generator.cache_size()),
			utils::abbreviate_number(config.main_config.caches.segments_cache_size_limit));
		
		let thumbnail_generator = ArtifactCache::builder()
			.cache_dir(config.paths.thumbnail_cache_dir.clone())
			.task_pool(transcoding_task_pool.clone())
			.file_size_limit(config.main_config.caches.thumbnail_cache_size_limit)
			.build(ThumbnailGenerator::new(media_backend_factory.clone()))
			.await?;
		
		info!("Thumbnail cache contains {}B, {}B max",
			utils::abbreviate_number(thumbnail_generator.cache_size()),
			utils::abbreviate_number(config.main_config.caches.thumbnail_cache_size_limit));
		
		let thumbnail_sheet_generator = ArtifactCache::builder()
			.cache_dir(config.paths.thumbnail_sheet_cache_dir.clone())
			.task_pool(transcoding_task_pool.clone())
			.file_size_limit(config.main_config.caches.thumbnail_sheet_cache_size_limit)
			.build(ThumbnailSheetGenerator::new(media_backend_factory.clone()))
			.await?;
		
		info!("Thumbnail sheet cache contains {}B, {}B max",
			utils::abbreviate_number(thumbnail_sheet_generator.cache_size()),
			utils::abbreviate_number(config.main_config.caches.thumbnail_sheet_cache_size_limit));
		
		let video_metadata_cache = MediaMetadataCache::new();
		
		Ok(Self {
			config,
			
			libraries,
			auth_manager,
			user_watch_histories,
			video_metadata_cache,
			
			hls_segment_generator,
			thumbnail_generator,
			thumbnail_sheet_generator,
		})
	}
}
