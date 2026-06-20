use std::sync::{Arc, Mutex};

use crate::config::ServerConfig;
use crate::web_server::auth::{AuthManager, AuthSecrets};
use crate::web_server::libraries::Libraries;
use crate::web_server::media_backend_factory::MediaBackendFactory;
use crate::web_server::services::artifact_cache::ArtifactCache;
use crate::web_server::services::hls_segment_service::HlsSegmentGenerator;
use crate::web_server::services::task_pool::TaskPool;
use crate::web_server::services::thumbnail_service::ThumbnailGenerator;
use crate::web_server::services::thumbnail_sheet_service::ThumbnailSheetGenerator;
use crate::web_server::metadata_cache::FileMetadataCache;
use crate::web_server::services::{hls_segment_service, scaled_thumbnail_service, subtitle_service, thumbnail_service, thumbnail_sheet_service, transcription_service};
use crate::web_server::services::scaled_thumbnail_service::ScaledThumbnailGenerator;
use crate::web_server::services::subtitle_service::TranscodedSubtitleGenerator;
use crate::web_server::services::transcription_service::AutoTranscriptionGenerator;
use crate::web_server::watch_history::UserWatchHistories;

pub struct ServerState {
	pub config: ServerConfig,
	
	pub libraries: Libraries,
	pub auth_manager: AuthManager,
	pub user_watch_histories: Arc<Mutex<UserWatchHistories>>,
	pub metadata_cache: FileMetadataCache,
	
	pub media_backend_factory: Arc<MediaBackendFactory>,
	
	pub hls_segment_generator: ArtifactCache<HlsSegmentGenerator>,
	pub thumbnail_generator: ArtifactCache<ThumbnailGenerator>,
	pub scaled_thumbnail_generator: ArtifactCache<ScaledThumbnailGenerator>,
	pub thumbnail_sheet_generator: ArtifactCache<ThumbnailSheetGenerator>,
	pub transcoded_subtitle_generator: ArtifactCache<TranscodedSubtitleGenerator>,
	pub auto_subtitle_generator: Option<ArtifactCache<AutoTranscriptionGenerator>>,
}

impl ServerState {
	pub async fn init(config: ServerConfig) -> anyhow::Result<Self> {
		let secrets_dir = config.paths.data_dir.join("secrets");
		tokio::fs::create_dir_all(&secrets_dir).await?;
		
		let libraries = Libraries::from_config(config.load_libraries_config().await?);
		
		let auth_secrets = AuthSecrets::load_from_file(&secrets_dir.join("auth-secrets.json")).await?;
		let auth_manager = AuthManager::from_config(config.load_users_config().await?, auth_secrets);
		
		let user_watch_histories = UserWatchHistories::load(&auth_manager,
			config.paths.data_dir.join("watch-histories")).await?;
		
		let media_backend_factory = Arc::new(MediaBackendFactory::new(config.main_config.transcoding.backend)?);
		let transcoding_task_pool = Arc::new(TaskPool::new(config.main_config.transcoding.concurrent_tasks));
		
		let hls_segment_generator = hls_segment_service::init_service(
			&config,
			transcoding_task_pool.clone(),
			media_backend_factory.clone(),
		).await?;
		
		let thumbnail_generator = thumbnail_service::init_service(
			&config,
			transcoding_task_pool.clone(),
			media_backend_factory.clone(),
		).await?;

		let scaled_thumbnail_generator = scaled_thumbnail_service::init_service(&config).await?;

		let thumbnail_sheet_generator = thumbnail_sheet_service::init_service(
			&config,
			transcoding_task_pool.clone(),
			media_backend_factory.clone(),
		).await?;

		let transcoded_subtitle_generator = subtitle_service::init_service(&config).await?;
		let auto_subtitle_generator = transcription_service::init_service(&config).await?;

		let metadata_cache = FileMetadataCache::new();
		
		Ok(Self {
			config,
			
			libraries,
			auth_manager,
			user_watch_histories,
			metadata_cache,
			
			media_backend_factory,
			
			hls_segment_generator,
			thumbnail_generator,
			scaled_thumbnail_generator,
			thumbnail_sheet_generator,
			transcoded_subtitle_generator,
			auto_subtitle_generator,
		})
	}
}
