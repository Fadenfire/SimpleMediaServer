use cfg_if::cfg_if;

use crate::media_manipulation::backends::{BackendFactory, VideoBackend};
use crate::media_manipulation::backends;

pub struct MediaBackendFactory;

impl MediaBackendFactory {
	pub fn new() -> Self {
		Self
	}
}

impl BackendFactory for MediaBackendFactory {
	fn create_video_backend(&self) -> anyhow::Result<Box<impl VideoBackend + 'static>> {
		cfg_if! {
			if #[cfg(target_os = "macos")] {
				let video_backend = backends::video_toolbox::VideoToolboxVideoBackend::new();
			} else if #[cfg(target_os = "linux")] {
				let video_backend = backends::intel_quick_sync::QuickSyncVideoBackend::new()?;
			} else {
				let video_backend = backends::software::SoftwareVideoBackend::new();
			}
		}
		
		Ok(Box::new(video_backend))
	}
}
