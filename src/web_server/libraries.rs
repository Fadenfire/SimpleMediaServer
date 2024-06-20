use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::ServerConfig;
use crate::web_server::api_routes::error::ApiError;
use crate::web_server::web_utils;

pub struct Libraries {
	library_table: HashMap<String, Library>,
}

impl Libraries {
	pub async fn load(server_config: &ServerConfig) -> anyhow::Result<Self> {
		let libraries_config = server_config.load_libraries_config().await?;
		
		let library_table = libraries_config.libraries.iter()
			.map(Clone::clone)
			.map(|lib| (lib.id.clone(), Library {
				id: lib.id,
				display_name: lib.display_name,
				root_path: lib.path,
			}))
			.collect();
		
		Ok(Self {
			library_table
		})
	}
	
	pub fn iter_libraries(&self) -> impl Iterator<Item = &Library> {
		self.library_table.values()
	}
	
	pub fn get_library(&self, library_id: &str) -> Option<&Library> {
		self.library_table.get(library_id)
	}
	
	pub fn resolve_library_and_path(&self, library_id: &str, path: &[&str]) -> Result<(&Library, PathBuf), ApiError> {
		let library = self.get_library(library_id).ok_or(ApiError::LibraryNotFound)?;
		let resolved_path = library.resolve_path(path).ok_or(ApiError::FileNotFound)?;
		
		Ok((library, resolved_path))
	}
	
	pub fn resolve_path(&self, library_id: &str, path: &[&str]) -> Result<PathBuf, ApiError> {
		self.resolve_library_and_path(library_id, path).map(|it| it.1)
	}
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Library {
	pub id: String,
	pub display_name: String,
	pub root_path: PathBuf,
}

impl Library {
	pub fn resolve_path(&self, path: &[&str]) -> Option<PathBuf> {
		let sanitized_path = web_utils::sanitize_path(path)?;
		
		Some(self.root_path.join(sanitized_path))
	}
}

pub fn reconstruct_library_path(library_id: &str, path: &[&str]) -> String {
	format!("{}/{}", library_id, path.join("/"))
}