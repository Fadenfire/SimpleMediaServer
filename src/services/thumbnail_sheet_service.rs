use std::path::PathBuf;

use bytes::Bytes;

use crate::media_manipulation::thumbnail_sheet;
use crate::media_manipulation::thumbnail_sheet::ThumbnailSheetParams;
use crate::services::artifact_cache::{ArtifactCache, ArtifactGenerator, FileValidityKey};

pub struct ThumbnailSheetGenerator;

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
		tokio::task::spawn_blocking(|| thumbnail_sheet::generate_sheet(media_path)).await.unwrap()
	}
}

pub async fn init_service(cache_dir: PathBuf) -> anyhow::Result<ArtifactCache<ThumbnailSheetGenerator>> {
	let service = ArtifactCache::new(ThumbnailSheetGenerator, cache_dir).await?
		.with_task_limit(2);
	
	Ok(service)
}
