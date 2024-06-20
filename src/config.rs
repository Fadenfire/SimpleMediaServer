use std::path::PathBuf;

use anyhow::Context;
use figment::Figment;
use figment::providers::{Env, Format, Serialized, Yaml};
use figment::value::magic::RelativePathBuf;
use serde::{Deserialize, Serialize};

use crate::utils;

#[derive(Debug, Clone)]
pub struct ServerConfig {
	pub main_config: GeneralConfig,
	pub libraries_config: LibrariesConfig,
	pub paths: ServerPaths,
}

impl ServerConfig {
	pub async fn load(main_config_path: PathBuf) -> anyhow::Result<Self> {
		let general_config: GeneralConfig = Figment::new()
			.merge(Serialized::defaults(GeneralConfig::default()))
			.merge(Yaml::file_exact(&main_config_path))
			.merge(Env::prefixed("MEDIA_SERVER_"))
			.extract()
			.context("Loading general config")?;
		
		let libraries_config: LibrariesConfig = match general_config.libraries_config {
			Some(ref path) => Figment::new()
				.merge(Yaml::file_exact(path.relative()))
				.extract()
				.context("Loading libraries config")?,
			None => LibrariesConfig::default(),
		};
		
		let paths = ServerPaths {
			data_dir: general_config.data_dir.clone(),
			cache_dir: general_config.cache_dir.clone(),
			
			transcoded_segments_cache_dir: general_config.cache_dir.join(&general_config.caches.segments_cache_dir),
			thumbnail_cache_dir: general_config.cache_dir.join(&general_config.caches.thumbnail_cache_dir),
			thumbnail_sheet_cache_dir:  general_config.cache_dir.join(&general_config.caches.thumbnail_sheet_cache_dir),
		};
		
		Ok(Self {
			main_config: general_config,
			libraries_config,
			paths,
		})
	}
}

#[derive(Debug, Clone)]
pub struct ServerPaths {
	pub data_dir: PathBuf,
	pub cache_dir: PathBuf,
	
	pub transcoded_segments_cache_dir: PathBuf,
	pub thumbnail_cache_dir: PathBuf,
	pub thumbnail_sheet_cache_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneralConfig {
	pub libraries_config: Option<RelativePathBuf>,
	pub data_dir: PathBuf,
	pub cache_dir: PathBuf,
	
	pub server: WebServerConfig,
	pub transcoding: TranscodingConfig,
	pub caches: CachesConfig,
}

impl Default for GeneralConfig {
	fn default() -> Self {
		Self {
			libraries_config: None,
			data_dir: PathBuf::from("data"),
			cache_dir: PathBuf::from("cache"),
			
			server: WebServerConfig::default(),
			transcoding: TranscodingConfig::default(),
			caches: CachesConfig::default(),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WebServerConfig {
	pub enable_http: bool,
	pub enable_https: bool,
	pub http_port: u16,
	pub https_port: u16,
}

impl Default for WebServerConfig {
	fn default() -> Self {
		Self {
			enable_http: true,
			enable_https: false,
			http_port: 8000,
			https_port: 8001,
		}
	}
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodingBackend {
	Software,
	VideoToolbox,
	IntelQuickSync,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TranscodingConfig {
	pub backend: TranscodingBackend,
	pub concurrent_tasks: usize,
}

impl Default for TranscodingConfig {
	fn default() -> Self {
		Self {
			backend: TranscodingBackend::Software,
			concurrent_tasks: 2,
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CachesConfig {
	pub segments_cache_dir: PathBuf,
	#[serde(deserialize_with = "utils::deserialize_suffixed_number")]
	pub segments_cache_size_limit: u64,
	
	pub thumbnail_cache_dir: PathBuf,
	#[serde(deserialize_with = "utils::deserialize_suffixed_number")]
	pub thumbnail_cache_size_limit: u64,
	
	pub thumbnail_sheet_cache_dir: PathBuf,
	#[serde(deserialize_with = "utils::deserialize_suffixed_number")]
	pub thumbnail_sheet_cache_size_limit: u64,
}

impl Default for CachesConfig {
	fn default() -> Self {
		Self {
			segments_cache_dir: PathBuf::from("transcoded-segments"),
			segments_cache_size_limit: u64::MAX,
			
			thumbnail_cache_dir: PathBuf::from("thumbnails"),
			thumbnail_cache_size_limit: u64::MAX,
			
			thumbnail_sheet_cache_dir: PathBuf::from("thumbnail-sheets"),
			thumbnail_sheet_cache_size_limit: u64::MAX,
		}
	}
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct LibrariesConfig {
	pub libraries: Vec<Library>
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct Library {
	pub id: String,
	pub display_name: String,
	pub path: PathBuf
}
