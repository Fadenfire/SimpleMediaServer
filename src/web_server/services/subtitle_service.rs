use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Context;
use bytes::Bytes;
use tracing::info;

use crate::config::ServerConfig;
use crate::media_manipulation::transcoding;
use crate::utils;
use crate::web_server::services::artifact_cache::{ArtifactCache, ArtifactGenerator, FileValidityKey};
use crate::web_server::services::task_pool::TaskPool;

#[derive(Debug, Clone)]
pub struct SubtitleParams {
	pub media_path: PathBuf,
	pub stream_index: usize,
}

pub async fn init_service(
	config: &ServerConfig,
) -> anyhow::Result<ArtifactCache<TranscodedSubtitleGenerator>> {
	let transcoded_subtitle_generator = ArtifactCache::builder()
		.cache_dir(config.paths.subtitles_cache_dir.clone())
		.task_pool(Arc::new(TaskPool::new(8)))
		.file_size_limit(config.main_config.caches.subtitles_cache_size_limit)
		.build(TranscodedSubtitleGenerator::new())
		.await?;

	info!("Transcoded subtitles cache contains {}B, {}B max",
		utils::abbreviate_number(transcoded_subtitle_generator.cache_size()),
		utils::abbreviate_number(config.main_config.caches.subtitles_cache_size_limit));

	Ok(transcoded_subtitle_generator)
}

pub struct TranscodedSubtitleGenerator;

impl TranscodedSubtitleGenerator {
	pub fn new() -> Self {
		Self
	}
}

impl ArtifactGenerator for TranscodedSubtitleGenerator {
	type Input = SubtitleParams;
	type ValidityKey = FileValidityKey;
	type Metadata = ();
	
	fn create_cache_key(&self, input: &Self::Input) -> String {
		let path_hash = blake3::hash(input.media_path.as_os_str().as_encoded_bytes()).to_hex();
		
		format!("{}_track_{}.vtt", path_hash, input.stream_index)
	}
	
	async fn create_validity_key(&self, input: &Self::Input) -> anyhow::Result<Self::ValidityKey> {
		FileValidityKey::from_file(&input.media_path).await
	}
	
	async fn generate_artifact(&self, input: Self::Input) -> anyhow::Result<(Bytes, Self::Metadata)> {
		info!("Transcoding subtitle for track {} of {:?}", input.stream_index, &input.media_path);
		let start_time = Instant::now();
		
		let data = tokio::task::spawn_blocking(move || {
			transcoding::subtitle::transcode_subtitle_to_webvtt(input.media_path, input.stream_index)
		}).await.context("Panic")??;
		
		info!("Generated subtitle in {:?}", start_time.elapsed());
		
		Ok((data, ()))
	}
}
