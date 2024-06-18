use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Context;
use bytes::Bytes;
use tracing::info;

use crate::media_manipulation::thumbnail;
use crate::web_server::media_backend_factory::MediaBackendFactory;
use crate::web_server::services::artifact_cache::{ArtifactGenerator, FileValidityKey};

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
	type ValidityKey = FileValidityKey;
	type Metadata = ();
	
	fn create_cache_key(&self, media_path: &Self::Input) -> String {
		format!("{}.jpg", blake3::hash(media_path.as_os_str().as_encoded_bytes()).to_hex())
	}
	
	async fn create_validity_key(&self, media_path: &Self::Input) -> anyhow::Result<Self::ValidityKey> {
		FileValidityKey::from_file(&media_path).await
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
