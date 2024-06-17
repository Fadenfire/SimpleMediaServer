<script lang="ts" context="module">
	export const TAP_SEEK_AMOUNT = 10;
</script>

<script lang="ts">
    import { escapePath, formatDuration, replayAnimations } from "$lib/utils";
    import type { SvelteMediaTimeRange } from "svelte/elements";
    import Timeline, { caclulateThumbnailSheetOffset } from "./Timeline.svelte";
    import FeatherIcon from "../FeatherIcon.svelte";
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import Spinner from "./Spinner.svelte";
    import { VideoBackend } from "./video_backend";

	export let mediaInfo: MediaInfo;
	
	let videoElement: HTMLVideoElement;
	let videoPaused = true;
	let videoEnded = false;
	let videoDuration = mediaInfo.duration;
	let videoCurrentTime = 0;
	let videoBuffered: SvelteMediaTimeRange[] = [];
	
	let playerElement: HTMLElement;
	let bottomControlsElement: HTMLElement;
	let scrubbingTime: number | null = null;
	
	let tapBackIndicatorElement: HTMLElement | undefined;
	let tapForwardIndicatorElement: HTMLElement | undefined;
	
	const mobile = window.matchMedia("(pointer: coarse)").matches;
	
	$: videoInfo = mediaInfo.video_info;
	
	// Player Backend
	
	let playerBackend: VideoBackend | undefined;
	
	$: if (videoElement) {
		playerBackend?.detach();
		
		playerBackend = new VideoBackend(videoElement, mediaInfo);
		playerBackend.attachHls();
	}
	
	// Thumbnail Sheet
	
	let mounted = true;
	let lastPath: string | undefined;
	let thumbSheetUrl: string | undefined;
	
	$: if (mediaInfo.path != lastPath) {
		lastPath = mediaInfo.path;
		
		if (thumbSheetUrl !== undefined) URL.revokeObjectURL(thumbSheetUrl);
		thumbSheetUrl = undefined;
		
		if (mediaInfo.video_info !== null) {
			fetch(`/api/thumbnail_sheet/${escapePath(mediaInfo.path)}`)
				.then(res => res.blob())
				.then(blob => {
					if (mounted) thumbSheetUrl = URL.createObjectURL(blob);
				});
		}
	}
	
	onMount(() => {
		return () => {
			mounted = false;
			
			if (thumbSheetUrl !== undefined) URL.revokeObjectURL(thumbSheetUrl);
			playerBackend?.detach();
		}
	});
	
	// Idleness
	
	let isIdle = true;
	let idleTimeout: number | undefined;
	
	function setIdle() {
		isIdle = true;
		clearTimeout(idleTimeout);
	}
	
	function resetIdleness() {
		isIdle = false;
		clearTimeout(idleTimeout);
		idleTimeout = setTimeout(() => isIdle = true, mobile ? 5000 : 3000);
	}
	
	// Show Controls
	
	let showControls = false;
	$: showControls = !isIdle || (!mobile && bottomControlsElement?.matches(":hover"));
	
	// Player Actions
	
	function playPause() {
		videoPaused = !videoPaused;
	}
	
	function gotoPrevVideo() {
		if (mediaInfo.prev_video !== null) goto(`../${encodeURIComponent(mediaInfo.prev_video)}/`, { replaceState: true });
	}
	
	function gotoNextVideo() {
		if (mediaInfo.next_video !== null) goto(`../${encodeURIComponent(mediaInfo.next_video)}/`, { replaceState: true });
	}
	
	function jump(amount: number) {
		const newTime = Math.max(0, Math.min(videoDuration, videoCurrentTime + amount));
		videoCurrentTime = newTime;
		
		if (videoElement.fastSeek) {
			videoElement.fastSeek(newTime);
		} else {
			videoElement.currentTime = newTime;
		}
	}
	
	// Fullscreen
	
	let isFullscreen = false;
	
	function toggleFullscreen() {
		if (document.fullscreenElement !== null) {
			document.exitFullscreen();
		} else {
			playerElement.requestFullscreen();
		}
	}
	
	function onFullscreenChange() {
		isFullscreen = document.fullscreenElement !== null;
	}
	
	// Tap Seeking
	
	let lastTapTime: number | null = null;
	let tapSeekBackAmount = 0;
	let tapSeekForwardAmount = 0;
	
	function playerClick(event: PointerEvent) {
		if (event.button != 0) return;
		
		if (!mobile) {
			playPause();
			
			event.preventDefault();
		} else {
			const isDoubleTap = lastTapTime !== null && Math.abs(event.timeStamp - lastTapTime) < 500;
			const boundingBox = playerElement.getBoundingClientRect();
			
			if (event.clientX < boundingBox.x + boundingBox.width * 0.3) {
				if (isDoubleTap) {
					jump(-TAP_SEEK_AMOUNT);
					
					tapSeekBackAmount += TAP_SEEK_AMOUNT;
					replayAnimations(tapBackIndicatorElement);
				} else {
					tapSeekBackAmount = 0;
					
					if (!isIdle) {
						setIdle();
						event.stopPropagation();
					}
				}
				
				lastTapTime = event.timeStamp;
			} else if (event.clientX > boundingBox.x + boundingBox.width * 0.7) {
				if (isDoubleTap) {
					jump(TAP_SEEK_AMOUNT);
					
					tapSeekForwardAmount += TAP_SEEK_AMOUNT;
					replayAnimations(tapForwardIndicatorElement);
				} else {
					tapSeekForwardAmount = 0;
					
					if (!isIdle) {
						setIdle();
						event.stopPropagation();
					}
				}
				
				lastTapTime = event.timeStamp;
			} else if (!isIdle) {
				setIdle();
				event.stopPropagation();
			}
			
			event.preventDefault();
		}
	}
	
	// Key Controls
	
	function onWindowKeyPressed(event: KeyboardEvent) {
		if (event.code === "Space") {
			playPause();
			
			resetIdleness();
		} else if (event.code === "ArrowLeft") {
			if (event.shiftKey) {
				jump(-60);
			} else if (event.altKey) {
				jump(-1);
			} else {
				jump(-10);
			}
		} else if (event.code === "ArrowRight") {
			if (event.shiftKey) {
				jump(60);
			} else if (event.altKey) {
				jump(1);
			} else {
				jump(10);
			}
		} else {
			return;
		}
		
		event.stopPropagation();
		event.preventDefault();
	}
	
	// Buffering
	
	let isBuffering = true;
	
	function onVideoLoadedData() {
		isBuffering = false;
	}
	
	function onVideoWaiting() {
		isBuffering = true;
	}
	
	function onVideoPlaying() {
		isBuffering = false;
	}
</script>

<svelte:window on:keydown={onWindowKeyPressed} />

<figure class="player-container" class:fullscreen={isFullscreen} bind:this={playerElement} on:pointermove={resetIdleness} on:pointerdown={resetIdleness} on:fullscreenchange={onFullscreenChange}>
	{#key mediaInfo}
		<!-- svelte-ignore a11y-media-has-caption -->
		<video
			bind:this={videoElement}
			bind:paused={videoPaused}
			bind:ended={videoEnded}
			bind:duration={videoDuration}
			bind:currentTime={videoCurrentTime}
			bind:buffered={videoBuffered}
			
			on:pointerdown={playerClick}
			on:loadeddata|once={onVideoLoadedData}
			on:waiting={onVideoWaiting}
			on:playing={onVideoPlaying}
			
			autoplay
		></video>
	{/key}
	
	{#if mobile && scrubbingTime !== null && videoInfo !== null && thumbSheetUrl !== undefined}
		{@const thumbOffset = caclulateThumbnailSheetOffset(scrubbingTime, videoInfo)}
		<div
			class="full-thumbnail"
			style="
				background-image: url({thumbSheetUrl});
				background-position: -{thumbOffset.spriteX * 100}% -{thumbOffset.spriteY * 100}%;
				background-size: {videoInfo.thumbnail_sheet_cols * 100}% {videoInfo.thumbnail_sheet_rows * 100}%;
			"
		></div>
	{/if}
	
	{#if isBuffering}
		<div class="spinner-container">
			<Spinner/>
		</div>
	{/if}
	
	{#if mobile || isFullscreen}
		<div class="top-controls hideable" class:hidden={!showControls}>
			<div class="control-row">
				<div class="control-element video-title">{mediaInfo.display_name}</div>
			</div>
		</div>
	{/if}
	
	{#if mobile}
		<div class="center-controls hideable" class:hidden={!showControls}>
			<button class="center-control-button" on:click={gotoPrevVideo} disabled={mediaInfo.prev_video === null}>
				<FeatherIcon name="skip-back" size="1em"/>
			</button>
			
			<button class="center-control-button play-pause" on:click={playPause}>
				{#if videoPaused || videoEnded}
					<FeatherIcon name="play" size="1em" style="transform: translateX(2px);"/>
				{:else}
					<FeatherIcon name="pause" size="1em"/>
				{/if}
			</button>
			
			<button class="center-control-button" on:click={gotoNextVideo} disabled={mediaInfo.next_video === null}>
				<FeatherIcon name="skip-forward" size="1em"/>
			</button>
		</div>
		
		{#if tapSeekBackAmount > 0}
			<div class="tap-seek-indicator" style="left: var(--video-player-seek-indicator-size);" bind:this={tapBackIndicatorElement}>
				<FeatherIcon name="rewind" size="3em"/>
				{tapSeekBackAmount} seconds
			</div>
		{/if}
		
		{#if tapSeekForwardAmount > 0}
			<div class="tap-seek-indicator" style="right: var(--video-player-seek-indicator-size);" bind:this={tapForwardIndicatorElement}>
				<FeatherIcon name="fast-forward" size="3em"/>
				{tapSeekForwardAmount} seconds
			</div>
		{/if}
	{/if}
	
	<div bind:this={bottomControlsElement} class="bottom-controls hideable" class:hidden={!showControls}>
		{#if !mobile}
			<Timeline
				mediaInfo={mediaInfo}
				thumbSheetUrl={thumbSheetUrl}
				mobile={false}
				
				videoElement={videoElement}
				bind:videoPaused={videoPaused}
				videoCurrentTime={videoCurrentTime}
				videoDuration={videoDuration}
				videoBuffered={videoBuffered}
				
				bind:scrubbingTime={scrubbingTime}
			/>
			
			<div class="control-row">
				<button class="control-button" on:click={gotoPrevVideo} disabled={mediaInfo.prev_video === null}>
					<FeatherIcon name="skip-back" size="1em"/>
				</button>
				
				<button class="control-button" on:click={playPause}>
					{#if videoPaused || videoEnded}
						<FeatherIcon name="play" size="1em"/>
					{:else}
						<FeatherIcon name="pause" size="1em"/>
					{/if}
				</button>
				
				<button class="control-button" on:click={gotoNextVideo} disabled={mediaInfo.next_video === null}>
					<FeatherIcon name="skip-forward" size="1em"/>
				</button>
				
				<div class="control-element">{formatDuration(videoCurrentTime)} / {formatDuration(videoDuration)}</div>
				
				<div class="spacer"></div>

				<button class="control-button" on:click={toggleFullscreen}>
					{#if isFullscreen}
						<FeatherIcon name="minimize" size="1em"/>
					{:else}
						<FeatherIcon name="maximize" size="1em"/>
					{/if}
				</button>
			</div>
		{:else}
			<div class="control-row">
				<div class="control-element">{formatDuration(videoCurrentTime)} / {formatDuration(videoDuration)}</div>
				
				<div class="spacer"></div>
				
				<button class="control-button" on:click={toggleFullscreen}>
					{#if isFullscreen}
						<FeatherIcon name="minimize" size="1em"/>
					{:else}
						<FeatherIcon name="maximize" size="1em"/>
					{/if}
				</button>
			</div>
			
			<Timeline
				mediaInfo={mediaInfo}
				thumbSheetUrl={thumbSheetUrl}
				mobile={true}
				
				videoElement={videoElement}
				bind:videoPaused={videoPaused}
				videoCurrentTime={videoCurrentTime}
				videoDuration={videoDuration}
				videoBuffered={videoBuffered}
				
				bind:scrubbingTime={scrubbingTime}
			/>
		{/if}
	</div>
</figure>

<style lang="scss">
	.player-container {
		width: 100%;
		height: 100%;
		background-color: black;
		position: relative;
		touch-action: manipulation;
		
		&.fullscreen {
			touch-action: none;
		}
	}
	
	video {
		width: 100%;
		height: 100%;
	}
	
	.hideable {
		visibility: visible;
		opacity: 1;
    	transition: visibility 0.5s, opacity 0.5s;
		
		&.hidden {
			visibility: hidden;
			opacity: 0;
		}
	}
	
	button {
		padding: 0;
		margin: 0;
		background: none;
		border-radius: 0;
		color: inherit;
		font-size: inherit;
		font-weight: inherit;
		text-decoration: none;
		border: none;
		cursor: pointer;
	}
	
	.full-thumbnail {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
	}
	
	.spinner-container {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		width: calc(var(--video-player-center-control-size) * 1.5);
		height: calc(var(--video-player-center-control-size) * 1.5);
		font-size: calc(var(--video-player-center-control-icon-size) * 1.5);
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 50%;
		background-color: #0009;
	}
	
	.top-controls {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		background: linear-gradient(to bottom, rgba(black, 0.6), transparent);
		padding: 0 8px;
	}
	
	.center-controls {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 32px;
	}
	
	.bottom-controls {
		position: absolute;
		bottom: 0;
		left: 0;
		width: 100%;
		display: flex;
		flex-direction: column;
		background: linear-gradient(to top, rgba(black, 0.6), transparent);
		padding: 0 8px;
	}
	
	.control-row {
		display: flex;
		padding: 8px;
		gap: 8px;
		
		.spacer {
			flex: 1;
		}
	}
	
	.control-button {
		width: var(--video-player-control-size);
		height: var(--video-player-control-size);
		font-size: var(--video-player-control-size);
		
		&:disabled {
			color: var(--disabled-text-color);
		}
	}
	
	.center-control-button {
		width: var(--video-player-center-control-size);
		height: var(--video-player-center-control-size);
		font-size: var(--video-player-center-control-icon-size);
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 50%;
		background-color: #0009;
		
		&.play-pause {
			width: calc(var(--video-player-center-control-size) * 1.5);
			height: calc(var(--video-player-center-control-size) * 1.5);
			font-size: calc(var(--video-player-center-control-icon-size) * 1.5);
		}
		
		&:disabled {
			color: var(--disabled-text-color);
		}
	}
	
	.control-element {
		height: var(--video-player-control-size);
		line-height: var(--video-player-control-size);
		font-size: 12px;
		font-weight: 500;
		text-align: center;
	}
	
	.video-title {
		font-size: var(--video-player-control-size);
		margin: 0 auto;
	}
	
	@keyframes fadeOut {
		from {
			opacity: 1;
			visibility: visible;
		}
		
		to {
			opacity: 0;
			visibility: hidden;
		}
	}
	
	.tap-seek-indicator {
		position: absolute;
		top: 50%;
		transform: translate(-50%, -50%);
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		width: var(--video-player-seek-indicator-size);
		height: var(--video-player-seek-indicator-size);
		font-size: 12px;
		font-weight: 500;
		border-radius: 50%;
		background-color: #0009;
		animation: fadeOut 1s 0.5s forwards;
		pointer-events: none;
	}
</style>