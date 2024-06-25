<script lang="ts">
    import type { SvelteMediaTimeRange } from 'svelte/elements';
    import { VideoBackend } from './video_backend';
    import { onMount } from 'svelte';

	export let mediaInfo: ApiMediaInfo;
	
	export let playerBackend: VideoBackend | undefined;
	export let videoElement: HTMLVideoElement | undefined;
	export let videoPaused: boolean;
	export let videoEnded: boolean;
	export let videoDuration: number;
	export let videoCurrentTime: number;
	export let videoBuffered: SvelteMediaTimeRange[];
	export let videoBuffering: boolean;
	
	let innerPlayerBackend: VideoBackend | undefined;
	
	let innerVideoElement: HTMLVideoElement | undefined = undefined;
	let innerPaused = true;
	let innerEnded = false;
	let innerDuration = mediaInfo.duration;
	let innerCurrentTime = 0;
	let innerBuffered: SvelteMediaTimeRange[] = [];
	let innerBuffering = true;
	
	$: playerBackend = innerPlayerBackend;
	$: videoElement = innerVideoElement;
	$: videoPaused = innerPaused;
	$: videoEnded = innerEnded;
	$: videoDuration = innerDuration;
	$: videoCurrentTime = innerCurrentTime;
	$: videoBuffered = innerBuffered;
	$: videoBuffering = innerBuffering;
	
	function onVideoLoadedData() {
		innerBuffering = false;
	}
	
	function onVideoWaiting() {
		innerBuffering = true;
	}
	
	function onVideoPlaying() {
		innerBuffering = false;
	}
	
	$: if (innerVideoElement && !innerPlayerBackend) {
		innerPlayerBackend = new VideoBackend(innerVideoElement, mediaInfo);
		innerPlayerBackend.attachNative();
		
		if (mediaInfo.watch_progress && mediaInfo.watch_progress < mediaInfo.duration - 10) {
			innerVideoElement.currentTime = mediaInfo.watch_progress;
		}
		
		console.log("Attached player backend");
	}
	
	onMount(() => {
		return () => {
			innerPlayerBackend?.detach();
			
			console.log("Dettached player backend");
		};
	})
</script>

<!-- svelte-ignore a11y-media-has-caption -->
<video
	bind:this={innerVideoElement}
	bind:paused={innerPaused}
	bind:ended={innerEnded}
	bind:duration={innerDuration}
	bind:currentTime={innerCurrentTime}
	bind:buffered={innerBuffered}

	on:loadeddata|once={onVideoLoadedData}
	on:waiting={onVideoWaiting}
	on:playing={onVideoPlaying}

	autoplay
>
	
</video>

<style lang="scss">
	video {
		width: 100%;
		height: 100%;
	}
</style>
