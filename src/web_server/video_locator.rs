use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::utils;
use crate::web_server::api_error::ApiError;

pub const MEDIA_EXTENSIONS: &[&str] = &[
	"mp4",
	"mkv",
	"webm",
	"mov",
];

pub enum LocatedFile {
	File(PathBuf),
	Directory(PathBuf),
}

impl LocatedFile {
	pub fn file(self) -> Result<PathBuf, ApiError> {
		match self { 
			Self::File(path) => Ok(path),
			_ => Err(ApiError::FileNotFound),
		}
	}
}

pub async fn locate_video(path: &Path) -> Result<LocatedFile, ApiError> {
	if let Ok(meta) = tokio::fs::metadata(path).await {
		if meta.is_dir() {
			return Ok(LocatedFile::Directory(path.to_owned()));
		}
	}
	
	for ext in MEDIA_EXTENSIONS {
		let ext_path = utils::add_extension(&path, ext);
		
		if tokio::fs::try_exists(&ext_path).await? {
			return Ok(LocatedFile::File(ext_path));
		}
	}
	
	Err(ApiError::FileNotFound)
}

pub fn is_video(path: &Path) -> bool {
	path
		.extension()
		.and_then(OsStr::to_str)
		.is_some_and(|ext| MEDIA_EXTENSIONS.contains(&ext))
}