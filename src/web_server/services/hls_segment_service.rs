use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use bytes::Bytes;

use crate::media_manipulation::transcoding;
use crate::media_manipulation::transcoding::TranscodingOptions;
use crate::web_server::media_backend_factory::MediaBackendFactory;
use crate::web_server::services::artifact_cache::{ArtifactCache, ArtifactGenerator, FileValidityKey};

pub const SEGMENT_DURATION: i64 = 5;

pub struct SegmentParams {
	pub media_path: PathBuf,
	pub segment_index: usize,
}

pub struct HlsSegmentGenerator {
	media_backend_factory: Arc<MediaBackendFactory>,
}

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
		let backend_factory = self.media_backend_factory.clone();
		
		let start_time = input.segment_index as i64 * SEGMENT_DURATION;
		let time_range = start_time..(start_time + SEGMENT_DURATION);
		
		let data = tokio::task::spawn_blocking(move || {
			transcoding::transcode_segment(TranscodingOptions {
				backend_factory: backend_factory.as_ref(),
				media_path: input.media_path,
				time_range,
				target_video_height: 1080,
				target_video_framerate: 60,
				video_bitrate: 12_000_000,
				audio_bitrate: 160_000,
			})
		}).await.context("Panic")??;
		
		Ok((data, ()))
	}
}

pub async fn init_service(
	cache_dir: PathBuf,
	media_backend_factory: Arc<MediaBackendFactory>,
) -> anyhow::Result<ArtifactCache<HlsSegmentGenerator>> {
	let generator = HlsSegmentGenerator {
		media_backend_factory,
	};
	
	let service = ArtifactCache::new(generator, cache_dir).await?
		.with_task_limit(2);
	
	Ok(service)
}
