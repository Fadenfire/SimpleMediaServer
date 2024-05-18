use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::LibrariesConfig;
use crate::utils;
use crate::web_server::api_routes::error::ApiError;

pub struct Libraries {
	library_table: HashMap<String, Library>,
}

impl Libraries {
	pub fn new(libraries_config: &LibrariesConfig) -> Self {
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
	
	pub fn split_library_path<'s, 'p>(&'s self, library_path: &'p str) -> Result<(&'s Library, &'p str), ApiError> {
		let (library_id, path) = library_path.split_once('/').ok_or(ApiError::NotFound)?;
		let library = self.get_library(library_id).ok_or(ApiError::LibraryNotFound)?;
		
		Ok((library, path))
	}
	
	pub fn resolve_library_and_path(&self, library_path: &str) -> Result<(&Library, PathBuf), ApiError> {
		let (library, path) = self.split_library_path(library_path)?;
		let resolved_path = library.resolve_path(path).ok_or(ApiError::FileNotFound)?;
		
		Ok((library, resolved_path))
	}
	
	pub fn resolve_path(&self, library_path: &str) -> Result<PathBuf, ApiError> {
		self.resolve_library_and_path(library_path).map(|it| it.1)
	}
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Library {
	pub id: String,
	pub display_name: String,
	pub root_path: PathBuf,
}

impl Library {
	pub fn resolve_path(&self, path: &str) -> Option<PathBuf> {
		let sanitized_path = utils::sanitize_path(path)?;
		
		Some(self.root_path.join(sanitized_path))
	}
}