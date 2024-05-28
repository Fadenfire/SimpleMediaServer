use std::path::PathBuf;

use bytes::Bytes;

use crate::media_manipulation::thumbnail;
use crate::services::artifact_cache::{ArtifactCache, ArtifactGenerator, FileValidityKey};

pub struct ThumbnailGenerator;

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
		let data = tokio::task::spawn_blocking(|| thumbnail::extract_thumbnail(media_path)).await.unwrap()?;
		
		Ok((data, ()))
	}
}

pub async fn init_service(cache_dir: PathBuf) -> anyhow::Result<ArtifactCache<ThumbnailGenerator>> {
	let service = ArtifactCache::new(ThumbnailGenerator, cache_dir).await?
		.with_task_limit(4);
	
	Ok(service)
}
