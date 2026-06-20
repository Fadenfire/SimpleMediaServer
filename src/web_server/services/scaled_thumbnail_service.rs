use std::io::Cursor;
use std::sync::Arc;

use bytes::Bytes;
use image::{GenericImageView, ImageReader};
use tracing::info;

use crate::config::ServerConfig;
use crate::utils;
use crate::web_server::services::artifact_cache::{ArtifactCache, ArtifactGenerator};
use crate::web_server::services::task_pool::TaskPool;

pub const TARGET_WIDTH: u32 = 640;
pub const TARGET_HEIGHT: u32 = 360;

const WEBP_QUALITY: f32 = 80.0;

pub async fn init_service(
	config: &ServerConfig,
) -> anyhow::Result<ArtifactCache<ScaledThumbnailGenerator>> {
	let scaled_thumbnail_generator = ArtifactCache::builder()
		.cache_dir(config.paths.scaled_thumbnail_cache_dir.clone())
		.task_pool(Arc::new(TaskPool::new(8)))
		.file_size_limit(config.main_config.caches.scaled_thumbnail_cache_size_limit)
		.build(ScaledThumbnailGenerator::new())
		.await?;

	info!("Scaled thumbnail cache contains {}B, {}B max",
		utils::abbreviate_number(scaled_thumbnail_generator.cache_size()),
		utils::abbreviate_number(config.main_config.caches.scaled_thumbnail_cache_size_limit));

	Ok(scaled_thumbnail_generator)
}

pub struct ScaledThumbnailGenerator;

impl ScaledThumbnailGenerator {
	pub fn new() -> Self {
		Self
	}
}

impl ArtifactGenerator for ScaledThumbnailGenerator {
	type Input = Bytes;
	type ValidityKey = String;
	type Metadata = ();
	
	fn create_cache_key(&self, thumbnail_data: &Self::Input) -> String {
		format!("{}.webp", blake3::hash(&thumbnail_data).to_hex())
	}
	
	async fn create_validity_key(&self, thumbnail_data: &Self::Input) -> anyhow::Result<Self::ValidityKey> {
		Ok(blake3::hash(&thumbnail_data).to_hex().to_string())
	}
	
	async fn generate_artifact(&self, thumbnail_data: Self::Input) -> anyhow::Result<(Bytes, Self::Metadata)> {
		let mut image = ImageReader::new(Cursor::new(thumbnail_data))
			.with_guessed_format()?
			.decode()?;
		
		let (width, height) = image.dimensions();
		
		if width > TARGET_WIDTH || height > TARGET_HEIGHT {
			image = image.thumbnail(TARGET_WIDTH, TARGET_HEIGHT);
		}
		
		let encoder = webp::Encoder::from_image(&image)
			.map_err(|err| anyhow::anyhow!("webp error: {}", err))?;
		
		let encoded_image = encoder.encode(WEBP_QUALITY);
		let output_bytes = encoded_image.to_vec().into();
		
		Ok((output_bytes, ()))
	}
}
