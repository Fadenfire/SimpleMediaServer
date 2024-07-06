use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use bytes::Bytes;

use crate::media_manipulation::transcoding;
use crate::media_manipulation::transcoding::TranscodingOptions;
use crate::web_server::api_routes::error::ApiError;
use crate::web_server::media_backend_factory::MediaBackendFactory;
use crate::web_server::services::artifact_cache::{ArtifactGenerator, FileValidityKey};
use crate::web_server::video_metadata::{Dimension, VideoMetadata};

pub const SEGMENT_DURATION: i64 = 5;

pub const QUALITY_LEVELS: &[HlsQualityLevel] = &[
	HlsQualityLevel {
		id: "1080p_12M",
		target_video_height: 1080,
		video_bitrate: 12_000_000,
		audio_bitrate: 192_000,
	},
	HlsQualityLevel {
		id: "720p_8M",
		target_video_height: 720,
		video_bitrate: 8_000_000,
		audio_bitrate: 192_000,
	},
	HlsQualityLevel {
		id: "480p_2M",
		target_video_height: 480,
		video_bitrate: 2_000_000,
		audio_bitrate: 128_000,
	},
	HlsQualityLevel {
		id: "360p_1M",
		target_video_height: 360,
		video_bitrate: 1_000_000,
		audio_bitrate: 96_000,
	},
];

pub fn get_quality_level(id: &str) -> Result<HlsQualityLevel, ApiError> {
	QUALITY_LEVELS.iter()
		.find(|lvl| lvl.id == id)
		.map(HlsQualityLevel::clone)
		.ok_or(ApiError::UnknownQualityLevel)
}

#[derive(Debug, Clone)]
pub struct HlsQualityLevel {
	pub id: &'static str,
	pub target_video_height: u32,
	pub video_bitrate: usize,
	pub audio_bitrate: usize,
}

impl HlsQualityLevel {
	pub fn max_bandwidth(&self) -> usize {
		self.video_bitrate + self.audio_bitrate + 16_000
	}
	
	pub fn supported(&self, video_metadata: &VideoMetadata) -> bool {
		self.target_video_height <= video_metadata.video_size.height
	}
	
	pub fn output_width(&self, source_size: &Dimension) -> u32 {
		transcoding::calculate_output_width(source_size.width, source_size.height, self.target_video_height)
	}
}

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
	type ValidityKey = FileValidityKey;
	type Metadata = ();
	
	fn create_cache_key(&self, input: &Self::Input) -> String {
		let path_hash = blake3::hash(input.media_path.as_os_str().as_encoded_bytes()).to_hex();
		
		format!("{}_{}_s{}.ts", path_hash, input.quality_level.id, input.segment_index)
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
				target_video_height: input.quality_level.target_video_height,
				// target_video_framerate: 60,
				video_bitrate: input.quality_level.video_bitrate,
				audio_bitrate: input.quality_level.audio_bitrate,
			})
		}).await.context("Panic")??;
		
		Ok((data, ()))
	}
}
