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
		
		subtitleTrack: number = $state(-1);
		
		subtitlesEnabled() {
			return this.subtitleTrack >= 0;
		}
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
	import { iso6393 } from "iso-639-3";

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
	
	// Set up video state
	
	const videoState = new VideoElementState();
	videoState.duration = mediaInfo.duration;
	
	provideState(videoState);
	
	let innerCurrentTime = $state(0.0);
	$effect(() => { videoState.currentTime = innerCurrentTime; });
	
	// Subtitles
	
	$effect(() => {
		if (videoState.videoElement === undefined) return;
		
		const textTracks = videoState.videoElement.textTracks;
		
		for (let i = 0; i < textTracks.length; i++) {
			textTracks[i].mode = i == videoState.subtitleTrack ? "showing" : "hidden";
		}
	});	
	
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
	
	function onVideoLoadedData(this: HTMLVideoElement) {
		if (initialLoad) {
			initialLoad = false;
			videoState.isBuffering = false;
		}
		
		// Move subtitle position
		
		for (const track of this.textTracks) {
			if (track.cues === null) return;
			
			for (const cue of track.cues) {
				(cue as VTTCue).line = -3;
			}
		}
	}
	
	function onVideoWaiting() {
		videoState.isBuffering = true;
	}
	
	function onVideoPlaying() {
		videoState.isBuffering = false;
	}
	
	// On mount callback
	
	let mounted = false;
	
	onMount(() => {
		mounted = true;
		
		if (videoState.videoElement === undefined) throw Error("Video element is undefined");
		
		// Set up backend
		
		videoState.playerBackend = new VideoBackend(videoState.videoElement, mediaInfo);
		videoState.playerBackend.currentLevelIndex.set(NATIVE_LEVEL_INDEX);
		
		// Seek if time override is provided
		
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
		
		// Request thumbnail sheet
		
		if (videoState.thumbSheetUrl !== undefined) URL.revokeObjectURL(videoState.thumbSheetUrl);
		videoState.thumbSheetUrl = undefined;
		
		if (mediaInfo.video_info !== null) {
			fetch(`/api/thumbnail_sheet/${escapePath(mediaInfo.full_path)}`)
				.then(res => res.blob())
				.then(blob => {
					if (mounted) videoState.thumbSheetUrl = URL.createObjectURL(blob);
				});
		}
		
		// Set up media session
		
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
		
		// Unmount callback
		
		return () => {
			mounted = false;
			
			if (videoState.thumbSheetUrl !== undefined) URL.revokeObjectURL(videoState.thumbSheetUrl);
			
			clearInterval(updateWatchProgressInterval);
			clearInterval(updateWatchTimeInterval);
			
			videoState.playerBackend?.detach();
			console.log("Dettached player backend");
			
			updateWatchProgress();
		};
	});
	
	function getSubtitleStreamLabel(stream: ApiSubtitleStream, index: number): string {
		if (stream.name !== null) return stream.name;
		
		if (stream.language !== null && stream.language !== "und") {
			const lang = iso6393.find(lang => lang.iso6393 === stream.language);
			
			if (lang !== undefined) return lang.name;
		}
		
		if (mediaInfo.subtitle_streams.length === 1) return "Default";
		
		return `Track ${index + 1}`;
	}
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
>
	{#each mediaInfo.subtitle_streams as subtitle_stream, index}
		<track
			kind="captions"
			srclang={subtitle_stream.language ?? undefined}
			label={getSubtitleStreamLabel(subtitle_stream, index)}
			src={`/api/subtitles/${escapePath(mediaInfo.full_path)}/track/${subtitle_stream.track_id}`}
		/>
	{/each}
</video>

<style lang="scss">
	video {
		width: 100%;
		height: 100%;
	}
	
	video::cue {
		font-size: 1.6rem;
	}
</style>
