<script lang="ts" module>
	export class VideoElementState {
		playerBackend: VideoBackend | undefined = $state();
		thumbSheetUrl: string | undefined = $state();
		videoElement: HTMLVideoElement | undefined = $state();
		
		isBuffering: boolean = $state(true);
		isPaused: boolean = $state(true);
		isEnded: boolean = $state(false);
		duration: number = $state(0.0);
		currentTime: number = $state(0.0);
		playbackRate: number = $state(1.0);
		buffered: SvelteMediaTimeRange[] = $state([]);
	}
</script>

<script lang="ts">
	import type { SvelteMediaTimeRange } from 'svelte/elements';
    import { NATIVE_LEVEL_INDEX, VideoBackend } from './video_backend';
    import { onMount } from 'svelte';
    import { escapePath, splitLibraryPath } from '$lib/utils';
    import { page } from '$app/stores';
    import { invalidate } from '$app/navigation';
    import { jumpToVideo } from './video_utils';

	interface Props {
		mediaInfo: ApiFileInfo;
		provideState: (state: VideoElementState) => void;
	}

	let {
		mediaInfo: initialMediaInfo,
		provideState,
	}: Props = $props();
	
	// Prevent media info from being changed
	const mediaInfo = initialMediaInfo;
	
	const videoState = new VideoElementState();
	videoState.duration = mediaInfo.duration;
	
	provideState(videoState);
	
	let innerCurrentTime = $state(0.0);
	$effect(() => { videoState.currentTime = innerCurrentTime; });
	
	let mounted = false;	
	
	// Watch Progress
	
	let watchTime = $state(0);
	
	function updateWatchProgress() {
		if (!videoState.playerBackend) return;
		if (watchTime < Math.min(2.0, videoState.duration / 20)) return;
		
		const [library_id, media_path] = splitLibraryPath(mediaInfo.full_path);
		
		const msg: UpdateWatchProgressParams = {
			library_id,
			media_path,
			new_watch_progress: Math.floor(videoState.currentTime),
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
	
	const updateWatchTimeInterval = setInterval(() => {
		if (!videoState.isPaused && !videoState.isEnded && !videoState.isBuffering) {
			watchTime += 1;
		}
	}, 1000);
	
	// Buffering
	
	let initialLoad = true;
	
	function onVideoLoadedData() {
		if (!initialLoad) return;
		
		initialLoad = false;
		videoState.isBuffering = false;
	}
	
	function onVideoWaiting() {
		videoState.isBuffering = true;
	}
	
	function onVideoPlaying() {
		videoState.isBuffering = false;
	}
	
	onMount(() => {
		mounted = true;
		
		if (videoState.videoElement === undefined) throw Error("Video element is undefined");
		
		videoState.playerBackend = new VideoBackend(videoState.videoElement, mediaInfo);
		videoState.playerBackend.currentLevelIndex.set(NATIVE_LEVEL_INDEX);
		
		const seekOverride: number | undefined = $page.state?.videoPlayerSeekTo;
		
		if (seekOverride !== undefined) {
			videoState.currentTime = seekOverride;
			videoState.videoElement.currentTime = seekOverride;
		} else if (mediaInfo.watch_progress && mediaInfo.watch_progress < mediaInfo.duration - 10) {
			videoState.currentTime = mediaInfo.watch_progress;
			videoState.videoElement.currentTime = mediaInfo.watch_progress;
		}
		
		videoState.isBuffering = true;
		console.log("Attached player backend");
		
		if (videoState.thumbSheetUrl !== undefined) URL.revokeObjectURL(videoState.thumbSheetUrl);
		videoState.thumbSheetUrl = undefined;
		
		if (mediaInfo.video_info !== null) {
			fetch(`/api/thumbnail_sheet/${escapePath(mediaInfo.full_path)}`)
				.then(res => res.blob())
				.then(blob => {
					if (mounted) videoState.thumbSheetUrl = URL.createObjectURL(blob);
				});
		}
		
		if ("mediaSession" in navigator) {
			navigator.mediaSession.metadata = new MediaMetadata({
				title: mediaInfo.display_name,
				artist: mediaInfo.artist ?? undefined,
				artwork: [
					{
						src: escapePath(mediaInfo.thumbnail_path),
					}
				],
			});
			
			navigator.mediaSession.setActionHandler("play", () => {
				videoState.videoElement?.play();
			});
			
			navigator.mediaSession.setActionHandler("pause", () => {
				videoState.videoElement?.pause();
			});
			
			if (mediaInfo.next_video !== null) {
				navigator.mediaSession.setActionHandler("nexttrack", () => {
					jumpToVideo(mediaInfo.next_video);
				});
			}
			
			if (mediaInfo.prev_video !== null) {
				navigator.mediaSession.setActionHandler("previoustrack", () => {
					jumpToVideo(mediaInfo.prev_video);
				});
			}
		}
		
		return () => {
			mounted = false;
			
			if (videoState.thumbSheetUrl !== undefined) URL.revokeObjectURL(videoState.thumbSheetUrl);
			
			clearInterval(updateWatchProgressInterval);
			clearInterval(updateWatchTimeInterval);
			
			videoState.playerBackend?.detach();
			console.log("Dettached player backend");
			
			updateWatchProgress();
		};
	})
</script>

<svelte:window onbeforeunload={updateWatchProgress}/>

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
