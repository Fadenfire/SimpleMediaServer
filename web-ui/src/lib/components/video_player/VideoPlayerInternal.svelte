<script lang="ts" module>
	export const AUTO_SUBTITLE_TRACK_ID = "auto_subtitle_track";
	
	export const NO_SUBTITLE_TRACK_INDEX = -1;
	export const AUTO_SUBTITLE_TRACK_INDEX = -2;
	
	const AUTO_SUBTITLE_SEGMENT_LENGTH = 120;

	export class VideoPlayerState {
		mediaInfo: ApiFileInfo;
		thumbSheetUrl: string | undefined = $state();
		videoState: VideoState = $state(new VideoState());
		
		subtitleTrack: number = $state(NO_SUBTITLE_TRACK_INDEX);
		
		constructor(mediaInfo: ApiFileInfo) {
			this.mediaInfo = mediaInfo;
			this.videoState.duration = mediaInfo.duration;
		}
		
		subtitlesEnabled() {
			return this.subtitleTrack != NO_SUBTITLE_TRACK_INDEX;
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
	import { WebVTTParser } from 'webvtt-parser';

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
	
	function subtitleIdFromIndex(index: number): string {
		if (index == AUTO_SUBTITLE_TRACK_INDEX) {
			return AUTO_SUBTITLE_TRACK_ID;
		}
		
		return `subtitle_track_${index}`;
	}
	
	function updateSubtitleVisibility(textTracks: TextTrackList, selectedTrack: number) {		
		for (const track of textTracks) {
			track.mode = track.id === subtitleIdFromIndex(selectedTrack) ? "showing" : "hidden";
		}
	}
	
	function transformCue(cue: TextTrackCue) {
		(cue as VTTCue).line = -3;
	}
	
	function transformSubtitleTrack(track: TextTrack) {
		Array.from(track.cues ?? []).forEach(transformCue)
	}
	
	$effect(() => {
		if (videoState.videoElement === undefined) return;
		
		updateSubtitleVisibility(
			videoState.videoElement.textTracks,
			playerState.subtitleTrack
		);
	});

	// Auto Subtitles
	//
	// Auto subtitles are generated lazily by the backend in segments and are
	//  only fetched once the auto subtitle track is selected. Each segment's
	//  cues use absolute timestamps, so they can be appended to a single track.

	let loadedAutoSegments = new Set<number>();

	function getAutoSubtitleTrack(): TextTrack | undefined {
		const textTracks = videoState.videoElement?.textTracks;
		if (textTracks === undefined) return undefined;

		for (const track of textTracks) {
			if (track.id === AUTO_SUBTITLE_TRACK_ID) return track;
		}

		return undefined;
	}

	async function loadAutoSubtitleSegment(segmentIndex: number) {
		if (segmentIndex < 0) return;
		if (segmentIndex * AUTO_SUBTITLE_SEGMENT_LENGTH >= mediaInfo.duration) return;
		if (loadedAutoSegments.has(segmentIndex)) return;

		loadedAutoSegments.add(segmentIndex);

		try {
			const res = await fetch(
				`/api/auto_subtitles/${escapePath(mediaInfo.full_path)}/segment/${segmentIndex}.vtt`
			);

			if (!res.ok) throw new Error(`Got status ${res.status}`);
			
			const text = await res.text();

			const track = getAutoSubtitleTrack();
			if (track === undefined) throw new Error("Auto subtitle track is missing");

			const parser = new WebVTTParser();
			const vttData = parser.parse(text);
						
			for (const cue of vttData.cues) {
				const webCue = new VTTCue(cue.startTime, cue.endTime, cue.text);
				transformCue(webCue);
				
				track.addCue(webCue);
			}

			loadedAutoSegments.add(segmentIndex);
		} catch (err) {
			// Allow the segment to be retried later
			loadedAutoSegments.delete(segmentIndex);
			console.error(`Failed to load auto subtitle segment ${segmentIndex}`, err);
		}
	}

	// Only changes when crossing a segment boundary, so the effect below
	//  doesn't re-run on every time update.
	let currentAutoSegment = $derived(Math.floor(videoState.currentTime / AUTO_SUBTITLE_SEGMENT_LENGTH));

	$effect(() => {
		if (playerState.subtitleTrack !== AUTO_SUBTITLE_TRACK_INDEX) return;

		loadAutoSubtitleSegment(currentAutoSegment)
			.then(() => loadAutoSubtitleSegment(currentAutoSegment + 1));
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
						src: escapePath(mediaInfo.full_thumbnail_path),
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
		
		// Subtitles
		
		const textTracks = videoState.videoElement.textTracks;
		
		textTracks.addEventListener("addtrack", (event) => {
			// Hide externally added text tracks as they're added
			updateSubtitleVisibility(textTracks, playerState.subtitleTrack);

			if (event.track?.id === AUTO_SUBTITLE_TRACK_ID) {
				loadedAutoSegments.clear();

				if (playerState.subtitleTrack === AUTO_SUBTITLE_TRACK_INDEX) {
					loadAutoSubtitleSegment(currentAutoSegment)
						.then(() => loadAutoSubtitleSegment(currentAutoSegment + 1));
				}
			}
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
		subtitleStreams={[
			...mediaInfo.subtitle_streams.map(stream => {
				return {
					id: subtitleIdFromIndex(stream.track_id),
					language: stream.language ?? undefined,
					src: `/api/subtitles/${escapePath(mediaInfo.full_path)}/track/${stream.track_id}.vtt`,
				};
			}),
			...(mediaInfo.has_auto_subtitles ? [{
				id: AUTO_SUBTITLE_TRACK_ID,
				language: undefined,
				src: undefined,
			}] : []),
		]}
		onSubtitleLoad={transformSubtitleTrack}
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
