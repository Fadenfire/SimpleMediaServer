use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

use crate::utils;

pub const MEDIA_EXTENSIONS: &[&str] = &[
	"mp4",
	"mkv",
	"webm",
	"mov",
];

pub async fn locate_video(path: &Path) -> io::Result<PathBuf> {
	if tokio::fs::try_exists(path).await? {
		return Ok(path.to_owned());
	}
	
	for ext in MEDIA_EXTENSIONS {
		let mut ext_path = path.to_owned();
		utils::add_extension(&mut ext_path, ext);
		
		if tokio::fs::try_exists(&ext_path).await? {
			return Ok(ext_path);
		}
	}
	
	Err(io::Error::from(io::ErrorKind::NotFound))
}

pub fn is_video(path: &Path) -> bool {
	path
		.extension()
		.and_then(OsStr::to_str)
		.is_some_and(|ext| MEDIA_EXTENSIONS.contains(&ext))
}