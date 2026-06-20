use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::config::ServerConfig;
use crate::media_manipulation::transcription;
use crate::utils;
use crate::web_server::services::artifact_cache::{ArtifactCache, ArtifactGenerator, FileValidityKey};
use crate::web_server::services::task_pool::TaskPool;
use anyhow::Context;
use bytes::Bytes;
use parakeet_rs::{ExecutionConfig, ParakeetTDT};
use tracing::info;

pub async fn init_service(
	config: &ServerConfig,
) -> anyhow::Result<Option<ArtifactCache<AutoTranscriptionGenerator>>> {
	let Some(model_path) = config.main_config.transcription.parakeet_model.clone() else { return Ok(None) };
	
	info!("Loading transcription model");
	let trans_config = config.main_config.transcription.clone();
	
	let model = tokio::task::spawn_blocking(move || {
		let model_config = ExecutionConfig::new()
			.with_intra_threads(trans_config.model_threads);
		
		ParakeetTDT::from_pretrained(model_path, Some(model_config))
	}).await??;
	
	info!("Loaded transcription model");
	
	let model = Arc::new(Mutex::new(model));
	
	let auto_subtitle_generator = ArtifactCache::builder()
		.cache_dir(config.paths.auto_subtitles_cache_dir.clone())
		.task_pool(Arc::new(TaskPool::new(1)))
		.file_size_limit(config.main_config.caches.auto_subtitles_cache_size_limit)
		.build(AutoTranscriptionGenerator::new(model))
		.await?;
	
	info!("Transcribed subtitle cache contains {}B, {}B max",
		utils::abbreviate_number(auto_subtitle_generator.cache_size()),
		utils::abbreviate_number(config.main_config.caches.auto_subtitles_cache_size_limit));
	
	Ok(Some(auto_subtitle_generator))
}

#[derive(Debug, Clone)]
pub struct AutoSubtitleParams {
	pub media_path: PathBuf,
	pub segment_index: usize,
}

pub struct AutoTranscriptionGenerator {
	model: Arc<Mutex<ParakeetTDT>>,
}

impl AutoTranscriptionGenerator {
	pub fn new(model: Arc<Mutex<ParakeetTDT>>) -> Self {
		Self {
			model,
		}
	}
}

const SEGMENT_LENGTH: f64 = 60.0;

impl ArtifactGenerator for AutoTranscriptionGenerator {
	type Input = AutoSubtitleParams;
	type ValidityKey = FileValidityKey;
	type Metadata = ();
	
	fn create_cache_key(&self, input: &Self::Input) -> String {
		let path_hash = blake3::hash(input.media_path.as_os_str().as_encoded_bytes()).to_hex();
		
		format!("{}_auto_s{}.vtt", path_hash, input.segment_index)
	}
	
	async fn create_validity_key(&self, input: &Self::Input) -> anyhow::Result<Self::ValidityKey> {
		FileValidityKey::from_file(&input.media_path).await
	}
	
	async fn generate_artifact(&self, input: Self::Input) -> anyhow::Result<(Bytes, Self::Metadata)> {
		info!("Transcribing subtitle for segment {} of {:?}", input.segment_index, &input.media_path);
		let start_time = Instant::now();
		
		let model_mutex = self.model.clone();
		
		let data = tokio::task::spawn_blocking(move || {
			let mut model = model_mutex.lock().unwrap();
			
			let seg_start = input.segment_index as f64 * SEGMENT_LENGTH;
			let time_bounds = seg_start..(seg_start + SEGMENT_LENGTH);
			
			transcription::transcribe(input.media_path, &mut model, time_bounds)
		}).await.context("Panic")??;
		
		info!("Transcribed segment in {:?}", start_time.elapsed());
		
		Ok((data.as_bytes().to_vec().into(), ()))
	}
}
