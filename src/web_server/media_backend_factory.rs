use crate::config::TranscodingBackend;
use crate::media_manipulation::backends::intel_quick_sync::QuickSyncVideoBackendFactory;
use crate::media_manipulation::backends::software::SoftwareVideoBackendFactory;
use crate::media_manipulation::backends::video_toolbox::VideoToolboxVideoBackendFactory;
use crate::media_manipulation::backends::{BackendFactory, VideoBackend};

pub struct MediaBackendFactory {
	backend_factory: Box<dyn BackendFactory + Send + Sync>,
}

impl MediaBackendFactory {
	pub fn new(backend_type: TranscodingBackend) -> anyhow::Result<Self> {
		Ok(Self {
			backend_factory: match backend_type {
				TranscodingBackend::Software => Box::new(SoftwareVideoBackendFactory::new()),
				TranscodingBackend::VideoToolbox => Box::new(VideoToolboxVideoBackendFactory::new()),
				TranscodingBackend::IntelQuickSync => Box::new(QuickSyncVideoBackendFactory::new()),
			},
		})
	}
}

impl BackendFactory for MediaBackendFactory {
	fn create_video_backend(&self) -> anyhow::Result<Box<dyn VideoBackend>> {
		self.backend_factory.create_video_backend()
	}
}
