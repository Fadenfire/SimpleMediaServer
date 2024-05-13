use std::path::{Path, PathBuf};
use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub struct ServerConfig {
	pub general_config: GeneralConfig,
	pub libraries_config: LibrariesConfig,
}

impl ServerConfig {
	pub async fn load(config_dir: PathBuf) -> anyhow::Result<Self> {
		let general_config = load_yaml_file(&config_dir.join("general.yml")).await
			.context("Loading general config")?;
		let libraries_config = load_yaml_file(&config_dir.join("libraries.yml")).await
			.context("Loading libraries config")?;
		
		Ok(Self {
			general_config,
			libraries_config,
		})
	}
}

async fn load_yaml_file<T: DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
	let data = tokio::fs::read(path).await?;
	let thing = serde_yaml::from_slice(&data)?;
	
	Ok(thing)
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
	pub server_http_port: u16,
	pub enable_https: bool,
	pub server_https_port: u16,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct LibrariesConfig {
	pub libraries: Vec<Library>
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct Library {
	pub id: String,
	pub display_name: String,
	pub path: PathBuf
}