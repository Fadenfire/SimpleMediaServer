use crate::config::ServerConfig;
use crate::media_manipulation::backends::BackendFactory;
use crate::media_manipulation::transcoding;
use crate::media_manipulation::transcoding::TranscodingOptions;
use crate::utils;
use crate::web_server::api_error::ApiError;
use crate::web_server::media_backend_factory::MediaBackendFactory;
use crate::web_server::media_metadata::{AdvancedMediaMetadata, Dimension, VideoMetadata};
use crate::web_server::services::artifact_cache::{self, ArtifactCache, ArtifactGenerator};
use crate::web_server::services::task_pool::TaskPool;
use anyhow::Context;
use bytes::Bytes;
use ffmpeg_next::codec;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

pub const SEGMENT_DURATION: f64 = 5.0;

pub const QUALITY_LEVELS: &[HlsQualityLevel] = &[
	HlsQualityLevel {
		id: "1080p_15M",
		target_video_height: 1080,
		video_codec: HlsVideoCodec::H264,
		video_bitrate: 15_000_000,
		audio_bitrate: 192_000,
	},
	HlsQualityLevel {
		id: "1080p_12M_HEVC",
		target_video_height: 1080,
		video_codec: HlsVideoCodec::HEVC,
		video_bitrate: 12_000_000,
		audio_bitrate: 192_000,
	},
	
	HlsQualityLevel {
		id: "720p_10M",
		target_video_height: 720,
		video_codec: HlsVideoCodec::H264,
		video_bitrate: 10_000_000,
		audio_bitrate: 192_000,
	},
	HlsQualityLevel {
		id: "720p_8M_HEVC",
		target_video_height: 720,
		video_codec: HlsVideoCodec::HEVC,
		video_bitrate: 8_000_000,
		audio_bitrate: 192_000,
	},
	HlsQualityLevel {
		id: "720p_2M_HEVC",
		target_video_height: 720,
		video_codec: HlsVideoCodec::HEVC,
		video_bitrate: 1_800_000,
		audio_bitrate: 128_000,
	},
	
	HlsQualityLevel {
		id: "480p_4M",
		target_video_height: 480,
		video_codec: HlsVideoCodec::H264,
		video_bitrate: 4_000_000,
		audio_bitrate: 192_000,
	},
	HlsQualityLevel {
		id: "480p_4M_HEVC",
		target_video_height: 480,
		video_codec: HlsVideoCodec::HEVC,
		video_bitrate: 4_000_000,
		audio_bitrate: 192_000,
	},
	HlsQualityLevel {
		id: "480p_1M_HEVC",
		target_video_height: 480,
		video_codec: HlsVideoCodec::HEVC,
		video_bitrate: 1_000_000,
		audio_bitrate: 128_000,
	},
	
	HlsQualityLevel {
		id: "360p_1M",
		target_video_height: 360,
		video_codec: HlsVideoCodec::H264,
		video_bitrate: 1_000_000,
		audio_bitrate: 96_000,
	},
	
	HlsQualityLevel {
		id: "240p_500k",
		target_video_height: 240,
		video_codec: HlsVideoCodec::H264,
		video_bitrate: 500_000,
		audio_bitrate: 96_000,
	},
];

pub fn get_quality_level(id: &str) -> Result<HlsQualityLevel, ApiError> {
	QUALITY_LEVELS.iter()
		.find(|lvl| lvl.id == id)
		.map(HlsQualityLevel::clone)
		.ok_or(ApiError::UnknownQualityLevel)
}

pub fn is_segment_index_valid(segment_index: usize, advanced_metadata: &AdvancedMediaMetadata) -> bool {
	segment_index as f64 * SEGMENT_DURATION <= advanced_metadata.ffmpeg_duration.as_secs_f64()
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HlsVideoCodec {
	H264,
	HEVC,
}

impl HlsVideoCodec {
	pub fn as_ffmpeg_codec(self) -> codec::Id {
		match self {
			HlsVideoCodec::H264 => codec::Id::H264,
			HlsVideoCodec::HEVC => codec::Id::HEVC,
		}
	}
	
	pub fn as_codec_string(self) -> &'static str {
		match self {
			HlsVideoCodec::H264 => "avc1.640033",
			HlsVideoCodec::HEVC => "hvc1.1.6.L153.B0",
		}
	}
}

pub const HLS_AUDIO_CODEC_STRING: &str = "mp4a.40.2";

#[derive(Debug, Clone)]
pub struct HlsQualityLevel {
	pub id: &'static str,
	pub target_video_height: u32,
	pub video_codec: HlsVideoCodec,
	pub video_bitrate: usize,
	pub audio_bitrate: usize,
}

impl HlsQualityLevel {
	pub fn max_bandwidth(&self) -> usize {
		self.video_bitrate + self.audio_bitrate + 16_000
	}
	
	pub fn supported(&self, video_metadata: &VideoMetadata, media_backend_factory: &impl BackendFactory) -> bool {
		self.target_video_height <= video_metadata.video_size.height &&
			media_backend_factory.supports_encoding_codec(self.video_codec.as_ffmpeg_codec())
	}
	
	pub fn output_width(&self, source_size: &Dimension) -> u32 {
		transcoding::calculate_output_width(source_size.width, source_size.height, self.target_video_height)
	}
}

pub async fn init_service(
	config: &ServerConfig,
	transcoding_task_pool: Arc<TaskPool>,
	media_backend_factory: Arc<MediaBackendFactory>,
) -> anyhow::Result<ArtifactCache<HlsSegmentGenerator>> {
	let hls_segment_generator = artifact_cache::builder()
		.cache_dir(config.paths.transcoded_segments_cache_dir.clone())
		.task_pool(transcoding_task_pool)
		.file_size_limit(config.main_config.caches.segments_cache_size_limit)
		.build(HlsSegmentGenerator::new(media_backend_factory))
		.await?;
	
	info!("HLS segments cache contains {}B, {}B max",
			utils::abbreviate_number(hls_segment_generator.cache_size()),
			utils::abbreviate_number(config.main_config.caches.segments_cache_size_limit));
	
	Ok(hls_segment_generator)
}

#[derive(Debug, Clone)]
pub struct SegmentParams {
	pub media_path: PathBuf,
	pub segment_index: usize,
	pub quality_level: HlsQualityLevel,
}

pub struct HlsSegmentGenerator {
	media_backend_factory: Arc<MediaBackendFactory>,
}

impl HlsSegmentGenerator {
	pub fn new(media_backend_factory: Arc<MediaBackendFactory>) -> Self {
		Self {
			media_backend_factory,
		}
	}
}

impl ArtifactGenerator for HlsSegmentGenerator {
	type Input = SegmentParams;
	type Metadata = ();

	async fn create_cache_key(&self, input: &Self::Input) -> anyhow::Result<String> {
		let file_hash = artifact_cache::create_fast_file_hash(&input.media_path).await?;

		Ok(format!("{}_{}_s{}.ts", file_hash, input.quality_level.id, input.segment_index))
	}

	async fn generate_artifact(&self, input: Self::Input) -> anyhow::Result<(Bytes, Self::Metadata)> {
		let backend_factory = self.media_backend_factory.clone();
		
		let start_time = Instant::now();
		
		let data = tokio::task::spawn_blocking(move || {
			let opts = TranscodingOptions {
				backend_factory: backend_factory.as_ref(),
				media_path: input.media_path,
				target_video_height: input.quality_level.target_video_height,
				video_codec: input.quality_level.video_codec.as_ffmpeg_codec(),
				// target_video_framerate: 60,
				video_bitrate: input.quality_level.video_bitrate,
				audio_bitrate: input.quality_level.audio_bitrate,
			};
			
			info!("Generating segment {} at {} for {:?}", input.segment_index, input.quality_level.id, &opts.media_path);
			
			let start_time = input.segment_index as f64 * SEGMENT_DURATION;
			let time_range = start_time..(start_time + SEGMENT_DURATION);
			
			transcoding::transcode_segment(opts, time_range)
		}).await.context("Panic")??;
		
		info!("Generated segment in {:?}", Instant::now() - start_time);
		
		Ok((data, ()))
	}
}
