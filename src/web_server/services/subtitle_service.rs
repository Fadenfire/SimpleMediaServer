use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Context;
use bytes::Bytes;
use tracing::info;

use crate::config::ServerConfig;
use crate::media_manipulation::transcoding;
use crate::utils;
use crate::web_server::services::artifact_cache::{self, ArtifactCache, ArtifactGenerator};
use crate::web_server::services::task_pool::TaskPool;

#[derive(Debug, Clone)]
pub struct SubtitleParams {
	pub media_path: PathBuf,
	pub stream_index: usize,
}

pub async fn init_service(
	config: &ServerConfig,
) -> anyhow::Result<ArtifactCache<TranscodedSubtitleGenerator>> {
	let transcoded_subtitle_generator = artifact_cache::builder()
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
	type Metadata = ();

	async fn create_cache_key(&self, input: &Self::Input) -> anyhow::Result<String> {
		let file_hash = artifact_cache::create_file_metadata_hash(&input.media_path).await?;

		Ok(format!("{}_track_{}.vtt", file_hash, input.stream_index))
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
