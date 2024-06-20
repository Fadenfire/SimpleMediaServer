import { escapePath } from "$lib/utils";
import Hls from "hls.js";

export enum SourceType {
	Native,
	Hls,
}

export class VideoBackend {
	videoElement: HTMLVideoElement;
	mediaInfo: MediaInfo;
	
	hls: Hls | undefined = undefined;
	
	currentSource: SourceType | undefined = undefined;
	
	constructor(videoElement: HTMLVideoElement, mediaInfo: MediaInfo) {
		this.videoElement = videoElement;
		this.mediaInfo = mediaInfo;
		
		if (Hls.isSupported()) {
			this.hls = new Hls();
			this.hls.loadSource(`/api/media/hls/${escapePath(this.mediaInfo.path)}/manifest`);
		}
		
		this.videoElement.addEventListener("error", () => this.#videoErrorHandler());
	}
	
	attachNative() {
		if (this.currentSource === SourceType.Native) return;
		
		const oldTime = this.videoElement.currentTime;
		
		this.hls?.detachMedia();
		
		this.videoElement.src = `/api/media/native/${escapePath(this.mediaInfo.path)}`;
		this.currentSource = SourceType.Native;
		
		this.videoElement.currentTime = oldTime;
	}
	
	attachHls() {
		if (this.currentSource === SourceType.Hls) return;
		
		const oldTime = this.videoElement.currentTime;
				
		if (this.hls !== undefined) {
			this.videoElement.src = "";
			
			this.hls.attachMedia(this.videoElement);
		} else {
			this.videoElement.src = `/api/media/hls/${escapePath(this.mediaInfo.path)}/manifest`;
		}
		
		this.currentSource = SourceType.Hls;
		
		this.videoElement.currentTime = oldTime;
	}
	
	// switchSource(newSource: SourceType) {
	// 	if (newSource === this.currentSource) return;
		
	// 	if (newSource === "native") {
	// 		this.attachNative();
	// 	} else if (newSource === "auto") {
	// 		this.attachHls();
	// 	} else {
	// 		throw new Error("Unknown source");
	// 	}
	// }
	
	detach() {
		this.hls?.detachMedia();
		this.hls?.destroy();
		this.hls = undefined;
		
		// this.videoElement.removeEventListener("error", this.#videoErrorHandler);
	}
	
	#videoErrorHandler() {
		const errorCode = this.videoElement.error?.code;
		
		if (this.currentSource === SourceType.Native && errorCode === MediaError.MEDIA_ERR_SRC_NOT_SUPPORTED) {
			this.attachHls();
		}
	}
}