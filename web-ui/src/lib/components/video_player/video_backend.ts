import { escapePath } from "$lib/utils";
import Hls from "hls.js";

export class VideoBackend {
	videoElement: HTMLVideoElement;
	mediaInfo: MediaInfo;
	supportsHls: boolean;
	
	hls: Hls | null | undefined = undefined;
	currentSource: string | undefined = undefined;
	
	constructor(videoElement: HTMLVideoElement, mediaInfo: MediaInfo) {
		this.videoElement = videoElement;
		this.mediaInfo = mediaInfo;
		
		this.supportsHls = Hls.isSupported();
	}
	
	attachNative() {
		if (this.hls === null) return;
		
		const oldTime = this.videoElement.currentTime;
		
		this.hls?.detachMedia();
		this.hls?.destroy();
		this.hls = null;
		
		this.videoElement.src = `/api/media/source/${escapePath(this.mediaInfo.path)}`;
		
		this.videoElement.currentTime = oldTime;
	}
	
	attachHls() {
		if (this.hls !== null && this.hls !== undefined) return;
		
		const oldTime = this.videoElement.currentTime;
		
		if (this.supportsHls) {
			this.videoElement.src = "";
			
			this.hls = new Hls();
			this.hls.loadSource(`/api/media/hls/${escapePath(this.mediaInfo.path)}/manifest`);
			this.hls.attachMedia(this.videoElement);
		} else {
			this.videoElement.src = `/api/media/hls/${escapePath(this.mediaInfo.path)}/manifest`;
		}
		
		this.videoElement.currentTime = oldTime;
	}
	
	switchSource(newSource: string) {
		if (newSource === this.currentSource) return;
		
		if (newSource === "native") {
			this.attachNative();
		} else if (newSource === "auto") {
			this.attachHls();
		} else {
			throw new Error("Unknown source");
		}
	}
	
	detach() {
		this.hls?.detachMedia();
		this.hls?.destroy();
		this.hls = undefined;
	}
}