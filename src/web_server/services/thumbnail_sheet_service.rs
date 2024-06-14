use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Context;
use bytes::Bytes;
use tracing::info;

use crate::media_manipulation::thumbnail_sheet;
use crate::media_manipulation::thumbnail_sheet::ThumbnailSheetParams;
use crate::web_server::media_backend_factory::MediaBackendFactory;
use crate::web_server::services::artifact_cache::{ArtifactCache, ArtifactGenerator, FileValidityKey};

pub struct ThumbnailSheetGenerator {
	media_backend_factory: Arc<MediaBackendFactory>,
}

impl ArtifactGenerator for ThumbnailSheetGenerator {
	type Input = PathBuf;
	type ValidityKey = FileValidityKey;
	type Metadata = ThumbnailSheetParams;
	
	fn create_cache_key(&self, media_path: &Self::Input) -> String {
		format!("{}.jpg", blake3::hash(media_path.as_os_str().as_encoded_bytes()).to_hex())
	}
	
	async fn create_validity_key(&self, media_path: &Self::Input) -> anyhow::Result<Self::ValidityKey> {
		FileValidityKey::from_file(&media_path).await
	}
	
	async fn generate_artifact(&self, media_path: Self::Input) -> anyhow::Result<(Bytes, Self::Metadata)> {
		let backend_factory = self.media_backend_factory.clone();
		
		info!("Generating thumbnail sheet for {:?}", &media_path);
		let start_time = Instant::now();
		
		let result = tokio::task::spawn_blocking(move || {
			thumbnail_sheet::generate_sheet(backend_factory.as_ref(), media_path)
		}).await.context("Panic")??;
		
		info!("Generated thumbnail sheet in {:?}", Instant::now() - start_time);
		
		Ok(result)
	}
}

pub async fn init_service(
	cache_dir: PathBuf,
	media_backend_factory: Arc<MediaBackendFactory>,
) -> anyhow::Result<ArtifactCache<ThumbnailSheetGenerator>> {
	let generator = ThumbnailSheetGenerator {
		media_backend_factory,
	};
	
	let service = ArtifactCache::new(generator, cache_dir).await?
		.with_task_limit(2);
	
	Ok(service)
}
