use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use anyhow::Context;
use bytes::Bytes;
use ffmpeg_next::codec;
use tracing::info;
use crate::media_manipulation::backends::BackendFactory;
use crate::media_manipulation::transcoding;
use crate::media_manipulation::transcoding::TranscodingOptions;
use crate::web_server::api_error::ApiError;
use crate::web_server::media_backend_factory::MediaBackendFactory;
use crate::web_server::services::artifact_cache::{ArtifactGenerator, FileValidityKey};
use crate::web_server::media_metadata::{Dimension, VideoMetadata};

pub const SEGMENT_DURATION: i64 = 5;

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
		audio_bitrate: 192_000,
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
		audio_bitrate: 192_000,
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

#[derive(Debug, Clone)]
pub struct SegmentParams {
	pub media_path: PathBuf,
	pub segment_index: SegmentIndex,
	pub quality_level: HlsQualityLevel,
}

#[derive(Debug, Clone)]
pub enum SegmentIndex {
	Init,
	Normal(usize),
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
		
		match input.segment_index {
			SegmentIndex::Init => format!("{}_{}_init.mp4", path_hash, input.quality_level.id),
			SegmentIndex::Normal(index) => format!("{}_{}_s{}.m4s", path_hash, input.quality_level.id, index),
		}
	}
	
	async fn create_validity_key(&self, input: &Self::Input) -> anyhow::Result<Self::ValidityKey> {
		FileValidityKey::from_file(&input.media_path).await
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
			
			match input.segment_index {
				SegmentIndex::Init => {
					info!("Generating init segment at {} for {:?}", input.quality_level.id, &opts.media_path);
					
					transcoding::generate_fmp4_init(opts)
				}
				SegmentIndex::Normal(index) => {
					info!("Generating segment {} at {} for {:?}", index, input.quality_level.id, &opts.media_path);
					
					let start_time = index as i64 * SEGMENT_DURATION;
					let time_range = start_time..(start_time + SEGMENT_DURATION);
					
					transcoding::generate_fmp4_segment(opts, time_range)
				}
			}
			
		}).await.context("Panic")??;
		
		info!("Generated segment in {:?}", Instant::now() - start_time);
		
		Ok((data, ()))
	}
}
