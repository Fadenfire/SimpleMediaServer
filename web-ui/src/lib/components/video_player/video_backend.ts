import { escapePath } from "$lib/utils";
import Hls, { Events } from "hls.js";

export enum SourceType {
	Native,
	Hls,
}

export class VideoBackend {
	videoElement: HTMLVideoElement;
	mediaInfo: ApiFileInfo;
	
	hls: Hls | undefined = undefined;
	
	currentSource: SourceType | undefined = undefined;
	
	constructor(videoElement: HTMLVideoElement, mediaInfo: ApiFileInfo) {
		this.videoElement = videoElement;
		this.mediaInfo = mediaInfo;
		
		if (Hls.isSupported()) {
			this.hls = new Hls();
			this.hls.loadSource(`/api/media/hls/${escapePath(this.mediaInfo.full_path)}/manifest`);
		}
		
		this.videoElement.addEventListener("error", () => this.#onVideoError());
		this.videoElement.addEventListener("loadeddata", () => this.#onVideoLoadedData());
	}
	
	attachNative() {
		if (this.currentSource === SourceType.Native) return;
		
		const oldTime = this.videoElement.currentTime;
		
		this.hls?.detachMedia();
		
		this.videoElement.src = `/api/media/native/${escapePath(this.mediaInfo.full_path)}`;
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
			this.videoElement.src = `/api/media/hls/${escapePath(this.mediaInfo.full_path)}/manifest`;
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
	
	#onVideoError() {
		const errorCode = this.videoElement.error?.code;
		
		if (this.currentSource === SourceType.Native && errorCode === MediaError.MEDIA_ERR_SRC_NOT_SUPPORTED) {
			this.attachHls();
		}
	}
	
	#onVideoLoadedData() {
		if (
			this.currentSource === SourceType.Native &&
			this.mediaInfo.video_info !== null &&
			(this.videoElement.videoWidth === 0 || this.videoElement.videoHeight === 0)
		) {
			// If there's supposed to be a video stream, but the video has no size then
			// it must have failed to decode so fallback to HLS.
			this.attachHls();
		}
	}
}