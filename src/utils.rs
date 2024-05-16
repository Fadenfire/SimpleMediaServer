use std::path::{Path, PathBuf};

pub fn add_extension(path: &mut PathBuf, extension: impl AsRef<Path>) {
	match path.extension() {
		Some(ext) => {
			let mut ext = ext.to_os_string();
			ext.push(".");
			ext.push(extension.as_ref());
			path.set_extension(ext)
		}
		None => path.set_extension(extension.as_ref()),
	};
}

pub fn sanitize_path(path: &str) -> Option<PathBuf> {
	let mut out_path = PathBuf::new();
	
	for component in path.split('/') {
		if component.starts_with("..") || component.contains('\\') {
			return None;
		}
		
		if component.is_empty() || component == "." {
			continue;
		}
		
		out_path.push(component);
	}
	
	if out_path.is_absolute() {
		return None;
	}
	
	if !out_path.components().all(|c| matches!(c, std::path::Component::Normal(_))) {
		return None;
	}
	
	Some(out_path)
}