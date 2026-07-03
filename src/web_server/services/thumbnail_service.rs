use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Context;
use bytes::Bytes;
use tracing::info;

use crate::config::ServerConfig;
use crate::media_manipulation::thumbnail;
use crate::utils;
use crate::web_server::media_backend_factory::MediaBackendFactory;
use crate::web_server::services::artifact_cache::{self, ArtifactCache, ArtifactGenerator};
use crate::web_server::services::task_pool::TaskPool;

pub async fn init_service(
	config: &ServerConfig,
	transcoding_task_pool: Arc<TaskPool>,
	media_backend_factory: Arc<MediaBackendFactory>,
) -> anyhow::Result<ArtifactCache<ThumbnailGenerator>> {
	let thumbnail_generator = artifact_cache::builder()
		.cache_dir(config.paths.thumbnail_cache_dir.clone())
		.task_pool(transcoding_task_pool)
		.file_size_limit(config.main_config.caches.thumbnail_cache_size_limit)
		.build(ThumbnailGenerator::new(media_backend_factory))
		.await?;

	info!("Thumbnail cache contains {}B, {}B max",
		utils::abbreviate_number(thumbnail_generator.cache_size()),
		utils::abbreviate_number(config.main_config.caches.thumbnail_cache_size_limit));

	Ok(thumbnail_generator)
}

pub struct ThumbnailGenerator {
	media_backend_factory: Arc<MediaBackendFactory>,
}

impl ThumbnailGenerator {
	pub fn new(media_backend_factory: Arc<MediaBackendFactory>) -> Self {
		Self {
			media_backend_factory,
		}
	}
}

impl ArtifactGenerator for ThumbnailGenerator {
	type Input = PathBuf;
	type Metadata = ();

	async fn create_cache_key(&self, media_path: &Self::Input) -> anyhow::Result<String> {
		let file_hash = artifact_cache::create_file_metadata_hash(media_path).await?;

		Ok(format!("{}.jpg", file_hash))
	}

	async fn generate_artifact(&self, media_path: Self::Input) -> anyhow::Result<(Bytes, Self::Metadata)> {
		let backend_factory = self.media_backend_factory.clone();
		
		info!("Generating thumbnail for {:?}", &media_path);
		let start_time = Instant::now();
		
		let data = tokio::task::spawn_blocking(move || {
			thumbnail::extract_thumbnail(backend_factory.as_ref(), media_path)
		}).await.context("Panic")??;
		
		info!("Generated thumbnail in {:?}", Instant::now() - start_time);
		
		Ok((data, ()))
	}
}
