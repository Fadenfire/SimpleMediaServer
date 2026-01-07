use std::path::PathBuf;
use std::time::Instant;

use anyhow::Context;
use bytes::Bytes;
use tracing::info;

use crate::media_manipulation::transcoding;
use crate::web_server::services::artifact_cache::{ArtifactGenerator, FileValidityKey};

#[derive(Debug, Clone)]
pub struct SubtitleParams {
	pub media_path: PathBuf,
	pub stream_index: usize,
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
