import { abbreviateNumber, escapePath } from "$lib/utils";
import Hls, { Events, type Level } from "hls.js";
import { get, writable, type Writable } from "svelte/store";

export enum SourceType {
	Native,
	Hls,
}

export enum LevelType {
	Native,
	HlsAuto,
	HlsManual,
}

export interface QualityLevel {
	id: number,
	levelType: LevelType,
	displayName: string,
	desc?: string,
	hlsLevelIndex?: number,
}

export const NATIVE_LEVEL_INDEX = 0;
export const HLS_AUTO_LEVEL_INDEX = 1;

export class VideoBackend {
	videoElement: HTMLVideoElement;
	mediaInfo: ApiFileInfo;
	nativeVideoURL: string;
	hlsManifestURL: string;
	
	hls: Hls | undefined;
	currentSource: SourceType | undefined;
	
	currentLevelIndex: Writable<number>;
	qualityLevels: Writable<QualityLevel[]>;
	
	constructor(videoElement: HTMLVideoElement, mediaInfo: ApiFileInfo) {
		this.videoElement = videoElement;
		this.mediaInfo = mediaInfo;
		
		this.nativeVideoURL = `/api/media/native/${escapePath(this.mediaInfo.full_path)}`;
		this.hlsManifestURL = `/api/media/hls/${escapePath(this.mediaInfo.full_path)}/manifest.m3u8`;
		
		this.currentLevelIndex = writable(-1);
		this.qualityLevels = writable(this.#createLevels());
		
		if (Hls.isSupported()) {
			this.hls = new Hls();
			this.hls.loadSource(this.hlsManifestURL);

			this.hls.on(Events.MANIFEST_PARSED, () => {
				// console.log(this.hls?.levels);
				
				this.qualityLevels.set(this.#createLevels());
			});
			
			this.hls.on(Events.LEVEL_SWITCHED, () => this.qualityLevels.set(this.#createLevels()));
		}
		
		this.videoElement.addEventListener("error", () => this.#onVideoError());
		this.videoElement.addEventListener("loadeddata", () => this.#onVideoLoadedData());
		
		this.currentLevelIndex.subscribe(newLevel => this.#switchLevel(newLevel));
	}
	
	detach() {
		this.hls?.detachMedia();
		this.hls?.destroy();
		this.hls = undefined;
		
		// this.videoElement.removeEventListener("error", this.#videoErrorHandler);
	}
	
	#attachNative() {
		if (this.currentSource === SourceType.Native) return;
		
		const oldTime = this.videoElement.currentTime;
		
		this.hls?.detachMedia();
		
		this.videoElement.src = this.nativeVideoURL;
		this.currentSource = SourceType.Native;
		
		this.videoElement.currentTime = oldTime;
	}
	
	#attachHls() {
		if (this.currentSource === SourceType.Hls) return;
		
		const oldTime = this.videoElement.currentTime;
				
		if (this.hls !== undefined) {
			this.videoElement.src = "";
			
			this.hls.attachMedia(this.videoElement);
		} else {
			this.videoElement.src = this.hlsManifestURL;
		}
		
		this.currentSource = SourceType.Hls;
		
		this.videoElement.currentTime = oldTime;
	}
	
	#switchLevel(newLevelIndex: number) {
		if (newLevelIndex == -1) return;
		
		const levels = get(this.qualityLevels);
		const newLevel = levels[newLevelIndex];
		
		if (newLevel.levelType == LevelType.Native) {
			this.#attachNative();
		} else if (newLevel.levelType == LevelType.HlsAuto) {
			this.#attachHls();
			
			if (this.hls) {
				this.hls.currentLevel = -1;
			}
		} else if (newLevel.levelType == LevelType.HlsManual) {
			this.#attachHls();
			
			if (this.hls) {
				this.hls.currentLevel = newLevel.hlsLevelIndex!;
				console.log(`Switching to HLS level ${newLevel.hlsLevelIndex}`);
			}
		}
		
		this.qualityLevels.set(this.#createLevels(newLevelIndex));
	}
	
	#createLevels(currentLevel?: number): QualityLevel[] {
		if (currentLevel === undefined) currentLevel = get(this.currentLevelIndex);
		
		let currentHlsLevelName: string | undefined;
		
		if (currentLevel == HLS_AUTO_LEVEL_INDEX && this.hls && this.hls.currentLevel >= 0) {
			currentHlsLevelName = genHlsLevelName(this.hls.levels[this.hls.currentLevel]);
		}
		
		let levelIndex = 0;
		
		const levels: QualityLevel[] = [
			{
				id: levelIndex++,
				levelType: LevelType.Native,
				displayName: "Native",
			},
			{
				id: levelIndex++,
				levelType: LevelType.HlsAuto,
				displayName: "Auto",
				desc: currentHlsLevelName,
			}
		];
		
		this.hls?.levels
			?.map((level, index) => [level, index] as [Level, number])
			?.reverse()
			?.forEach(([level, id]) => {
				levels.push({
					id: levelIndex++,
					levelType: LevelType.HlsManual,
					displayName: genHlsLevelName(level),
					hlsLevelIndex: id,
				});
			});
		
		return levels;
	}
	
	#onVideoError() {
		const errorCode = this.videoElement.error?.code;
		
		if (this.currentSource === SourceType.Native && errorCode === MediaError.MEDIA_ERR_SRC_NOT_SUPPORTED) {
			this.currentLevelIndex.set(HLS_AUTO_LEVEL_INDEX);
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
			this.currentLevelIndex.set(HLS_AUTO_LEVEL_INDEX);
		}
	}
}

function genHlsLevelName(level: Level): string {
	return `${level.height}p - ${abbreviateNumber(level.bitrate)}`;
}
