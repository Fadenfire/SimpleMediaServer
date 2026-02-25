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
		src: string;
	}
</script>

<!-- svelte-ignore state_referenced_locally -->
<script lang="ts">
	import type { SvelteMediaTimeRange } from 'svelte/elements';
    import { NATIVE_LEVEL_INDEX, VideoBackend } from './video_backend';
    import { onMount } from 'svelte';

	interface Props {
		mediaPath: string;
		subtitleStreams?: SubtitleStream[];
		provideState: (state: VideoState) => void;
		
		onVideoLoadedData?: (this: HTMLVideoElement) => void;
	}

	let {
		mediaPath,
		subtitleStreams,
		provideState,
		
		onVideoLoadedData: onVideoLoadedDataCallback,
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
		
		onVideoLoadedDataCallback?.call(this);
	}
	
	function onVideoWaiting() {
		videoState.isBuffering = true;
	}
	
	function onVideoPlaying() {
		videoState.isBuffering = false;
	}
	
	// On mount callback
	
	onMount(() => {
		if (videoState.videoElement === undefined) throw Error("Video element is undefined");
		
		// Set up backend
		
		videoState.playerBackend = new VideoBackend(videoState.videoElement, mediaPath);
		videoState.playerBackend.currentLevelIndex.set(NATIVE_LEVEL_INDEX);
		
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
>
	{#each subtitleStreams as subtitleStream}
		<track
			kind="captions"
			id={subtitleStream.id}
			srclang={subtitleStream.language}
			src={subtitleStream.src}
		/>
	{/each}
</video>

<style lang="scss">
	video {
		width: 100%;
		height: 100%;
	}
</style>
