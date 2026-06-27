<script lang="ts" module>
	export class VideoState {
		playerBackend: VideoBackend | undefined = $state();
		videoElement: HTMLVideoElement | undefined = $state();
		
		isBuffering: boolean = $state(true);
		isPaused: boolean = $state(true);
		isEnded: boolean = $state(false);
		duration: number = $state(0.0);
		currentTime: number = $state(0.0);
		playbackRate: number = $state(1.0);
		buffered: SvelteMediaTimeRange[] = $state([]);
	}
	
	export interface SubtitleStream {
		id: string;
		language: string | undefined;
		src: string | undefined;
	}
	
	const MAX_BUFFERING_TIME_MS = 1000;
</script>

<!-- svelte-ignore state_referenced_locally -->
<script lang="ts">
	import type { SvelteMediaTimeRange } from 'svelte/elements';
    import { HLS_AUTO_LEVEL_INDEX, NATIVE_LEVEL_INDEX, SourceType, VideoBackend } from './video_backend';
    import { onMount } from 'svelte';
    import type { HlsConfig } from 'hls.js';

	interface Props {
		mediaPath: string;
		subtitleStreams?: SubtitleStream[];
		hlsConfig?: Partial<HlsConfig>;
		provideState: (state: VideoState) => void;
		onSubtitleLoad?: (track: TextTrack) => void;
	}

	let {
		mediaPath,
		subtitleStreams,
		hlsConfig,
		provideState,
		onSubtitleLoad = () => {},
	}: Props = $props();
	
	// Set up video state
	
	const videoState = new VideoState();
	provideState(videoState);
	
	let innerCurrentTime = $state(0.0);
	$effect(() => { videoState.currentTime = innerCurrentTime; });
	
	// Buffering

	let initialLoad = true;

	function onVideoLoadedData(this: HTMLVideoElement) {
		if (initialLoad) {
			initialLoad = false;
			videoState.isBuffering = false;
		}
	}
	
	function onVideoWaiting() {
		videoState.isBuffering = true;
	}
	
	function onVideoPlaying() {
		videoState.isBuffering = false;
	}
	
	// Slow Loading Detection
	
	let slowLoadingTimeout: number | undefined;
	
	$effect(() => {
		if (videoState.isBuffering) {
			if (slowLoadingTimeout === undefined) {
				slowLoadingTimeout = setTimeout(() => {
					clearTimeout(slowLoadingTimeout);
					slowLoadingTimeout = undefined;
					
					if (videoState.playerBackend?.currentSource == SourceType.Native) {
						console.log("Video is loading slowly, switching to HLS for better performance");
						
						videoState.playerBackend?.currentLevelIndex.set(HLS_AUTO_LEVEL_INDEX);
					}
				}, MAX_BUFFERING_TIME_MS);
			}
		} else {
			clearTimeout(slowLoadingTimeout);
			slowLoadingTimeout = undefined;
		}
	});
	
	// On mount callback

	onMount(() => {
		if (videoState.videoElement === undefined) throw Error("Video element is undefined");

		// Set up backend

		const backend = new VideoBackend(videoState.videoElement, mediaPath, hlsConfig);

		// Re-add all subtitles tracks whenever we switch backends. This is done
		//  because hls.js clears all subtitle tracks when it's attached.
		backend.onMediaAttached = () => {
			const video = videoState.videoElement;
			if (!video) return;

			for (const elem of Array.from(video.querySelectorAll("track"))) {
				elem.remove();
			}

			for (const stream of subtitleStreams ?? []) {
				const trackElem = document.createElement("track");
				trackElem.kind = "captions";
				trackElem.id = stream.id;
				if (stream.language) trackElem.srclang = stream.language;
				
				if (stream.src !== undefined) {
					trackElem.src = stream.src;
					
					trackElem.addEventListener("load", () => {
						onSubtitleLoad(trackElem.track);
					});
				}
				
				video.appendChild(trackElem);
			}
		};

		videoState.playerBackend = backend;
		backend.currentLevelIndex.set(NATIVE_LEVEL_INDEX);

		videoState.isBuffering = true;
		console.log("Attached player backend");

		// Unmount callback

		return () => {
			videoState.playerBackend?.detach();
			console.log("Dettached player backend");
		};
	});
</script>

<!-- svelte-ignore a11y_media_has_caption -->
<video
	bind:this={videoState.videoElement}
	bind:paused={videoState.isPaused}
	bind:ended={videoState.isEnded}
	bind:duration={videoState.duration}
	bind:playbackRate={videoState.playbackRate}
	bind:buffered={videoState.buffered}

	bind:currentTime={innerCurrentTime}

	onloadeddata={onVideoLoadedData}
	onwaiting={onVideoWaiting}
	onplaying={onVideoPlaying}

	autoplay
></video>

<style lang="scss">
	video {
		width: 100%;
		height: 100%;
	}
</style>
