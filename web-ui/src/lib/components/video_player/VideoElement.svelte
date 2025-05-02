<script lang="ts">
	import type { SvelteMediaTimeRange } from 'svelte/elements';
    import { NATIVE_LEVEL_INDEX, VideoBackend } from './video_backend';
    import { onMount } from 'svelte';
    import { escapePath, splitLibraryPath } from '$lib/utils';
    import { page } from '$app/stores';
    import { invalidate } from '$app/navigation';

	interface Props {
		mediaInfo: ApiFileInfo;
		playerBackend?: VideoBackend | undefined;
		thumbSheetUrl?: string | undefined;
		videoBuffering?: boolean;
		videoElement: HTMLVideoElement | undefined;
		videoPaused: boolean;
		videoEnded: boolean;
		videoDuration: number;
		videoCurrentTime: number;
		videoBuffered: SvelteMediaTimeRange[];
	}

	let {
		mediaInfo,
		playerBackend = $bindable(),
		thumbSheetUrl = $bindable(),
		videoBuffering = $bindable(),
		videoElement = $bindable(),
		videoPaused = $bindable(),
		videoEnded = $bindable(),
		videoDuration = $bindable(),
		videoCurrentTime = $bindable(),
		videoBuffered = $bindable()
	}: Props = $props();
	
	let innerVideoElement: HTMLVideoElement | undefined = $state();
	let innerPaused = $state(true);
	let innerEnded = $state(false);
	let innerDuration = $state(mediaInfo.duration);
	let innerCurrentTime = $state(0);
	let innerBuffered: SvelteMediaTimeRange[] = $state([]);
	
	$effect(() => { videoElement = innerVideoElement; });
	$effect(() => { videoPaused = innerPaused; });
	$effect(() => { videoEnded = innerEnded; });
	$effect(() => { videoDuration = innerDuration; });
	$effect(() => { videoCurrentTime = innerCurrentTime; });
	$effect(() => { videoBuffered = innerBuffered; });
	
	let mounted = false;	
	
	// Watch Progress
	
	function updateWatchProgress() {
		if (!playerBackend) return;
		if (videoCurrentTime < Math.min(1.0, videoDuration / 20)) return;
		
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
	
	let initialLoad = true;
	
	function onVideoLoadedData() {
		if (!initialLoad) return;
		
		initialLoad = false;
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
		
		if (innerVideoElement === undefined) throw Error("Video element is undefined");
		
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

<svelte:window onbeforeunload={updateWatchProgress}/>

<!-- svelte-ignore a11y_media_has_caption -->
<video
	bind:this={innerVideoElement}
	bind:paused={innerPaused}
	bind:ended={innerEnded}
	bind:duration={innerDuration}
	bind:currentTime={innerCurrentTime}
	bind:buffered={innerBuffered}

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
