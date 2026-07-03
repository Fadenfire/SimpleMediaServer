use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::config::ServerConfig;
use crate::media_manipulation::transcription;
use crate::utils;
use crate::web_server::services::artifact_cache::{self, ArtifactCache, ArtifactGenerator};
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
	
	let auto_subtitle_generator = artifact_cache::builder()
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

const SEGMENT_LENGTH: f64 = 120.0;
const SEGMENT_OVERLAP: f64 = 15.0;

impl ArtifactGenerator for AutoTranscriptionGenerator {
	type Input = AutoSubtitleParams;
	type Metadata = ();

	async fn create_cache_key(&self, input: &Self::Input) -> anyhow::Result<String> {
		let file_hash = artifact_cache::create_file_metadata_hash(&input.media_path).await?;

		Ok(format!("{}_auto_s{}.vtt", file_hash, input.segment_index))
	}

	async fn generate_artifact(&self, input: Self::Input) -> anyhow::Result<(Bytes, Self::Metadata)> {
		info!("Transcribing subtitle for segment {} of {:?}", input.segment_index, &input.media_path);
		let start_time = Instant::now();
		
		let model_mutex = self.model.clone();
		
		let data = tokio::task::spawn_blocking(move || {
			let mut model = model_mutex.lock().unwrap();
			
			let seg_start = input.segment_index as f64 * SEGMENT_LENGTH;
			let time_bounds = seg_start..(seg_start + SEGMENT_LENGTH);
			
			transcription::transcribe(input.media_path, &mut model, time_bounds, SEGMENT_OVERLAP)
		}).await.context("Panic")??;
		
		info!("Transcribed segment in {:?}", start_time.elapsed());
		
		Ok((data.as_bytes().to_vec().into(), ()))
	}
}
