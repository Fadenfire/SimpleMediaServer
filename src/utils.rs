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