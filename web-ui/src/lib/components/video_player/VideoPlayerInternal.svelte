<script lang="ts" module>
	export class VideoPlayerState {
		mediaInfo: ApiFileInfo;
		thumbSheetUrl: string | undefined = $state();
		videoState: VideoState = $state(new VideoState());
		
		subtitleTrack: number = $state(-1);
		
		constructor(mediaInfo: ApiFileInfo) {
			this.mediaInfo = mediaInfo;
			this.videoState.duration = mediaInfo.duration;
		}
		
		subtitlesEnabled() {
			return this.subtitleTrack >= 0;
		}
	}
</script>

<!-- svelte-ignore state_referenced_locally -->
<script lang="ts">
    import { onMount } from 'svelte';
    import { escapePath, splitLibraryPath } from '$lib/utils';
    import { page } from '$app/stores';
    import { invalidate } from '$app/navigation';
    import { jumpToVideo } from './video_utils';
    import VideoElement, { VideoState } from './VideoElement.svelte';

	interface Props {
		mediaInfo: ApiFileInfo;
		provideState: (state: VideoPlayerState) => void;
	}

	let {
		mediaInfo: initialMediaInfo,
		provideState,
	}: Props = $props();
	
	// Prevent media info from being changed
	const mediaInfo = initialMediaInfo;
	
	// Set up video state
	
	const playerState = new VideoPlayerState(mediaInfo);
	provideState(playerState);
	
	const videoState = $derived(playerState.videoState);
	
	// Subtitles
	
	function makeSubtitleTrackId(track_id: number): string {
		return `subtitle_track_${track_id}`;
	}
	
	function updateSubtitleVisibility(textTracks: TextTrackList, selectedTrack: number) {		
		for (const track of textTracks) {
			track.mode = track.id === makeSubtitleTrackId(selectedTrack) ? "showing" : "hidden";
		}
	}
	
	$effect(() => {
		if (videoState.videoElement === undefined) return;
		
		updateSubtitleVisibility(
			videoState.videoElement.textTracks,
			playerState.subtitleTrack
		);
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
	
	function onVideoLoadedData(this: HTMLVideoElement) {
		for (const track of this.textTracks) {
			if (track.cues === null) return;
			
			for (const cue of track.cues) {
				(cue as VTTCue).line = -3;
			}
		}
	}
	
	// On mount callback
	
	let mounted = false;
	
	onMount(() => {
		mounted = true;
		
		if (videoState.videoElement === undefined) throw Error("Video element is undefined");
		
		// Seek if time override is provided
		
		const seekOverride: number | undefined = $page.state?.videoPlayerSeekTo;
		
		if (seekOverride !== undefined) {
			videoState.currentTime = seekOverride;
			videoState.videoElement.currentTime = seekOverride;
		} else if (mediaInfo.watch_progress && mediaInfo.watch_progress < mediaInfo.duration - 10) {
			videoState.currentTime = mediaInfo.watch_progress;
			videoState.videoElement.currentTime = mediaInfo.watch_progress;
		}
		
		// Request thumbnail sheet
		
		if (playerState.thumbSheetUrl !== undefined) URL.revokeObjectURL(playerState.thumbSheetUrl);
		playerState.thumbSheetUrl = undefined;
		
		if (mediaInfo.video_info !== null) {
			fetch(`/api/thumbnail_sheet/${escapePath(mediaInfo.full_path)}`)
				.then(res => res.blob())
				.then(blob => {
					if (mounted) playerState.thumbSheetUrl = URL.createObjectURL(blob);
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
		
		// Hide externally added text tracks
		
		const textTracks = videoState.videoElement.textTracks;
		
		textTracks.addEventListener("addtrack", () => {
			updateSubtitleVisibility(textTracks, playerState.subtitleTrack);
		});
				
		// Unmount callback
		
		return () => {
			mounted = false;
			
			if (playerState.thumbSheetUrl !== undefined) URL.revokeObjectURL(playerState.thumbSheetUrl);
			
			clearInterval(updateWatchProgressInterval);
			clearInterval(updateWatchTimeInterval);
			
			updateWatchProgress();
		};
	});
</script>

<svelte:window onbeforeunload={updateWatchProgress}/>

<div>
	<VideoElement
		mediaPath={mediaInfo.full_path}
		provideState={state => playerState.videoState = state}
		subtitleStreams={
			mediaInfo.subtitle_streams.map(stream => {
				return {
					id: makeSubtitleTrackId(stream.track_id),
					language: stream.language ?? undefined,
					src: `/api/subtitles/${escapePath(mediaInfo.full_path)}/track/${stream.track_id}`,
				};
			})
		}
		
		{onVideoLoadedData}
	/>
</div>

<style lang="scss">
	div {
		display: contents;
	}

	div :global(video::cue) {
		font-size: 1.6rem;
	}
</style>
