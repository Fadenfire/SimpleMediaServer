use std::path::PathBuf;

use bytes::Bytes;

use crate::media_manipulation::transcoding;
use crate::services::artifact_cache::{ArtifactCache, ArtifactGenerator, FileValidityKey};

pub const SEGMENT_DURATION: i64 = 5;

pub struct SegmentParams {
	pub media_path: PathBuf,
	pub segment_index: usize,
}

pub struct HlsSegmentGenerator;

impl ArtifactGenerator for HlsSegmentGenerator {
	type Input = SegmentParams;
	type ValidityKey = FileValidityKey;
	type Metadata = ();
	
	fn create_cache_key(&self, input: &Self::Input) -> String {
		format!("{}_{}.ts", blake3::hash(input.media_path.as_os_str().as_encoded_bytes()).to_hex(), input.segment_index)
	}
	
	async fn create_validity_key(&self, input: &Self::Input) -> anyhow::Result<Self::ValidityKey> {
		FileValidityKey::from_file(&input.media_path).await
	}
	
	async fn generate_artifact(&self, input: Self::Input) -> anyhow::Result<(Bytes, Self::Metadata)> {
		let data = tokio::task::spawn_blocking(move || {
			transcoding::transcode_segment(input.media_path, input.segment_index, SEGMENT_DURATION)
		}).await.unwrap()?;
		
		Ok((data, ()))
	}
}

pub async fn init_service(cache_dir: PathBuf) -> anyhow::Result<ArtifactCache<HlsSegmentGenerator>> {
	let service = ArtifactCache::new(HlsSegmentGenerator, cache_dir).await?
		.with_task_limit(2);
	
	Ok(service)
}
