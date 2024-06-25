use std::path::PathBuf;

use anyhow::Context;
use figment::Figment;
use figment::providers::{Env, Format, Serialized, Yaml};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::utils;

pub const GENERAL_CONFIG_NAME: &str = "general.yml";
pub const LIBRARIES_CONFIG_NAME: &str = "libraries.yml";
pub const USERS_CONFIG_NAME: &str = "users.yml";

#[derive(Debug, Clone)]
pub struct ServerConfig {
	pub main_config: GeneralConfig,
	pub paths: ServerPaths,
}

impl ServerConfig {
	pub async fn load(data_dir: PathBuf, cache_dir: PathBuf) -> anyhow::Result<Self> {
		let web_ui_dir = std::env::var_os("WEB_UI_DIR")
			.map(PathBuf::from)
			.unwrap_or_else(|| {
				let exe_path = std::env::current_exe().expect("Could not get location of current executable");
				exe_path.parent().unwrap().join("web-ui")
			});
		
		if !tokio::fs::try_exists(&web_ui_dir).await.unwrap_or(false) {
			panic!("Web UI assets directory does not exist");
		}
		
		let config_dir = data_dir.join("config");
		
		let general_config: GeneralConfig = Figment::new()
			.merge(Serialized::defaults(GeneralConfig::default()))
			.merge(Yaml::file_exact(&config_dir.join(GENERAL_CONFIG_NAME)))
			.merge(Env::prefixed("MEDIA_SERVER_"))
			.extract()
			.context("Loading general config")?;
		
		let paths = ServerPaths {
			data_dir: data_dir.clone(),
			cache_dir: cache_dir.clone(),
			web_ui_dir,
			config_dir,
			
			transcoded_segments_cache_dir: cache_dir.join(&general_config.caches.segments_cache_dir),
			thumbnail_cache_dir: cache_dir.join(&general_config.caches.thumbnail_cache_dir),
			thumbnail_sheet_cache_dir: cache_dir.join(&general_config.caches.thumbnail_sheet_cache_dir),
		};
		
		Ok(Self {
			main_config: general_config,
			paths,
		})
	}
	
	async fn load_config<T: DeserializeOwned + Default>(path: PathBuf) -> anyhow::Result<T> {
		if tokio::fs::try_exists(&path).await? {
			Figment::new()
				.merge(Yaml::file_exact(path))
				.extract()
				.context("Loading config")
		} else {
			Ok(T::default())
		}
	}
	
	pub async fn load_libraries_config(&self) -> anyhow::Result<LibrariesConfig> {
		Self::load_config(self.paths.config_dir.join(LIBRARIES_CONFIG_NAME)).await
	}
	
	pub async fn load_users_config(&self) -> anyhow::Result<UsersConfig> {
		Self::load_config(self.paths.config_dir.join(USERS_CONFIG_NAME)).await
	}
}

#[derive(Debug, Clone)]
pub struct ServerPaths {
	pub data_dir: PathBuf,
	pub cache_dir: PathBuf,
	pub web_ui_dir: PathBuf,
	pub config_dir: PathBuf,
	
	pub transcoded_segments_cache_dir: PathBuf,
	pub thumbnail_cache_dir: PathBuf,
	pub thumbnail_sheet_cache_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneralConfig {
	pub server: WebServerConfig,
	pub transcoding: TranscodingConfig,
	pub caches: CachesConfig,
}

impl Default for GeneralConfig {
	fn default() -> Self {
		Self {
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
#[serde(deny_unknown_fields)]
pub struct LibrariesConfig {
	pub libraries: Vec<LibraryConfig>,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LibraryConfig {
	pub id: String,
	pub display_name: String,
	pub path: PathBuf,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsersConfig {
	pub users: Vec<UserConfig>,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserConfig {
	pub id: String,
	pub display_name: String,
	pub username: String,
	pub password: String,
	pub allowed_libraries: Vec<String>,
}
