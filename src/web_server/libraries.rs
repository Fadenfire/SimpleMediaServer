use std::collections::HashMap;
use std::path::PathBuf;

use http::HeaderMap;
use relative_path::{RelativePath, RelativePathBuf};

use crate::config::LibrariesConfig;
use crate::web_server::api_routes::error::ApiError;
use crate::web_server::server_state::ServerState;
use crate::web_server::web_utils;

pub struct Libraries {
	library_table: HashMap<String, Library>,
}

impl Libraries {
	pub fn from_config(libraries_config: LibrariesConfig) -> Self {
		let library_table = libraries_config.libraries.iter()
			.map(Clone::clone)
			.map(|lib| (lib.id.clone(), Library {
				id: lib.id,
				display_name: lib.display_name,
				root_path: lib.path,
			}))
			.collect();
		
		Self {
			library_table
		}
	}
	
	pub fn iter_libraries(&self) -> impl Iterator<Item = &Library> {
		self.library_table.values()
	}
	
	pub fn get_library(&self, library_id: &str) -> Option<&Library> {
		self.library_table.get(library_id)
	}
	
	pub fn resolve_library_and_path(&self, library_id: &str, path: RelativePathBuf) -> Result<(&Library, PathBuf), ApiError> {
		let library = self.get_library(library_id).ok_or(ApiError::LibraryNotFound)?;
		let resolved_path = library.resolve_path(&path).ok_or(ApiError::FileNotFound)?;
		
		Ok((library, resolved_path))
	}
	
	pub fn resolve_path(&self, library_id: &str, path: RelativePathBuf) -> Result<PathBuf, ApiError> {
		self.resolve_library_and_path(library_id, path).map(|it| it.1)
	}
}

#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub struct Library {
	pub id: String,
	pub display_name: String,
	pub root_path: PathBuf,
}

impl Library {
	pub fn resolve_path(&self, path: &RelativePath) -> Option<PathBuf> {
		let sanitized_path = web_utils::sanitize_path(&path)?;
		
		Some(self.root_path.join(sanitized_path))
	}
}

pub fn resolve_library_and_path_with_auth<'a>(
	server_state: &'a ServerState,
	library_id: &str,
	path: RelativePathBuf,
	headers: &HeaderMap
) -> Result<(&'a Library, PathBuf), ApiError> {
	let user = server_state.auth_manager.lookup_from_headers(headers)?;
	
	if !user.can_see_library(library_id) {
		return Err(ApiError::LibraryNotFound);
	}
	
	server_state.libraries.resolve_library_and_path(library_id, path)
}

pub fn resolve_path_with_auth(
	server_state: &ServerState,
	library_id: &str,
	path: RelativePathBuf,
	headers: &HeaderMap
) -> Result<PathBuf, ApiError> {
	let user = server_state.auth_manager.lookup_from_headers(headers)?;
	
	if !user.can_see_library(library_id) {
		return Err(ApiError::LibraryNotFound);
	}
	
	server_state.libraries.resolve_path(library_id, path)
}