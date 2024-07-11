<script lang="ts">
    import type { SvelteMediaTimeRange } from 'svelte/elements';
    import { NATIVE_LEVEL_INDEX, VideoBackend } from './video_backend';
    import { onMount } from 'svelte';
    import { escapePath, splitLibraryPath } from '$lib/utils';
    import { page } from '$app/stores';
    import { invalidate } from '$app/navigation';

	export let mediaInfo: ApiFileInfo;
	
	export let playerBackend: VideoBackend | undefined = undefined;
	export let thumbSheetUrl: string | undefined = undefined;
	export let videoBuffering = true;
	
	export let videoElement: HTMLVideoElement;
	export let videoPaused: boolean;
	export let videoEnded: boolean;
	export let videoDuration: number;
	export let videoCurrentTime: number;
	export let videoBuffered: SvelteMediaTimeRange[];
	
	let innerVideoElement: HTMLVideoElement;
	let innerPaused = true;
	let innerEnded = false;
	let innerDuration = mediaInfo.duration;
	let innerCurrentTime = 0;
	let innerBuffered: SvelteMediaTimeRange[] = [];
	
	$: videoElement = innerVideoElement;
	$: videoPaused = innerPaused;
	$: videoEnded = innerEnded;
	$: videoDuration = innerDuration;
	$: videoCurrentTime = innerCurrentTime;
	$: videoBuffered = innerBuffered;
	
	let mounted = false;	
	
	// Watch Progress
	
	function updateWatchProgress() {
		if (!playerBackend) return;
		
		const [library_id, media_path] = splitLibraryPath(mediaInfo.full_path);
		
		const msg: UpdateWatchProgressParams = {
			library_id,
			media_path,
			new_watch_progress: Math.floor(videoCurrentTime),
		};
		
		fetch("/api/update_watch_progress", {
			method: "POST",
			body: JSON.stringify(msg)
		})
		.then(() => {
			invalidate(url => url.pathname == "/api/watch_history" ||
				url.pathname.startsWith(`/api/list_dir/${encodeURIComponent(library_id)}/`));
		});
	}
	
	const updateWatchProgressInterval = setInterval(updateWatchProgress, 60 * 1000);
	
	// Buffering
	
	function onVideoLoadedData() {
		videoBuffering = false;
	}
	
	function onVideoWaiting() {
		videoBuffering = true;
	}
	
	function onVideoPlaying() {
		videoBuffering = false;
	}
	
	onMount(() => {
		mounted = true;
		
		playerBackend = new VideoBackend(innerVideoElement, mediaInfo);
		playerBackend.currentLevelIndex.set(NATIVE_LEVEL_INDEX);
		
		const seekOverride: number | undefined = $page.state?.videoPlayerSeekTo;
		
		if (seekOverride !== undefined) {
			innerCurrentTime = seekOverride;
			innerVideoElement.currentTime = seekOverride;
		} else if (mediaInfo.watch_progress && mediaInfo.watch_progress < mediaInfo.duration - 10) {
			innerCurrentTime = mediaInfo.watch_progress;
			innerVideoElement.currentTime = mediaInfo.watch_progress;
		}
		
		videoBuffering = true;
		console.log("Attached player backend");
		
		if (thumbSheetUrl !== undefined) URL.revokeObjectURL(thumbSheetUrl);
		thumbSheetUrl = undefined;
		
		if (mediaInfo.video_info !== null) {
			fetch(`/api/thumbnail_sheet/${escapePath(mediaInfo.full_path)}`)
				.then(res => res.blob())
				.then(blob => {
					if (mounted) thumbSheetUrl = URL.createObjectURL(blob);
				});
		}
		
		return () => {
			mounted = false;
			
			if (thumbSheetUrl !== undefined) URL.revokeObjectURL(thumbSheetUrl);
			
			clearInterval(updateWatchProgressInterval);
			
			playerBackend?.detach();
			console.log("Dettached player backend");
			
			updateWatchProgress();
		};
	})
</script>

<svelte:window on:beforeunload={updateWatchProgress}/>

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
></video>

<style lang="scss">
	video {
		width: 100%;
		height: 100%;
	}
</style>
