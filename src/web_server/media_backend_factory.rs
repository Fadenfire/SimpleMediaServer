use crate::config::TranscodingBackend;
use crate::media_manipulation::backends::{BackendFactory, VideoBackend};
use crate::media_manipulation::backends::intel_quick_sync::QuickSyncVideoBackend;
use crate::media_manipulation::backends::software::SoftwareVideoBackend;
use crate::media_manipulation::backends::video_toolbox::VideoToolboxVideoBackend;

pub struct MediaBackendFactory {
	backend_type: TranscodingBackend,
}

impl MediaBackendFactory {
	pub fn new(backend_type: TranscodingBackend) -> anyhow::Result<Self> {
		Ok(Self {
			backend_type,
		})
	}
}

impl BackendFactory for MediaBackendFactory {
	fn create_video_backend(&self) -> anyhow::Result<Box<dyn VideoBackend>> {
		let video_backend: Box<dyn VideoBackend> = match self.backend_type {
			TranscodingBackend::Software => Box::new(SoftwareVideoBackend::new()),
			TranscodingBackend::VideoToolbox => Box::new(VideoToolboxVideoBackend::new()),
			TranscodingBackend::IntelQuickSync => Box::new(QuickSyncVideoBackend::new()?),
		};
		
		Ok(video_backend)
	}
}
