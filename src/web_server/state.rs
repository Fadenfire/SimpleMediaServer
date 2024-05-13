use std::collections::HashMap;

use crate::config::{LibrariesConfig, Library, ServerConfig};

pub struct ServerState {
	pub server_config: ServerConfig,
	pub libraries: Libraries,
}

impl ServerState {
	pub fn new(server_config: ServerConfig) -> Self {
		let libraries = Libraries::new(&server_config.libraries_config);
		
		Self {
			server_config,
			libraries,
		}
	}
}

pub struct Libraries {
	library_table: HashMap<String, Library>,
}

impl Libraries {
	pub fn new(libraries_config: &LibrariesConfig) -> Self {
		let library_table = libraries_config.libraries.iter()
			.map(|it| (it.id.clone(), it.clone()))
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
}