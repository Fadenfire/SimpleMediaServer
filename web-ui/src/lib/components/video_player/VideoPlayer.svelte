<script lang="ts" context="module">
	export const TAP_SEEK_AMOUNT = 10;
</script>

<script lang="ts">
    import { formatDuration, replayAnimations } from "$lib/utils";
    import type { SvelteMediaTimeRange } from "svelte/elements";
    import Timeline from "./Timeline.svelte";
    import FeatherIcon from "../FeatherIcon.svelte";
    import Spinner from "./Spinner.svelte";
    import { VideoBackend } from "./video_backend";
    import VideoElement from "./VideoElement.svelte";
    import PlayPauseButton from "./buttons/PlayPauseButton.svelte";
    import SkipButton from "./buttons/SkipButton.svelte";
    import FullscreenButton from "./buttons/FullscreenButton.svelte";
    import PreviewThumbnail from "./PreviewThumbnail.svelte";
    import ConnectionsButton from "./buttons/ConnectionsButton.svelte";

	export let mediaInfo: ApiMediaInfo;
	
	// Video properties
	let playerBackend: VideoBackend | undefined;
	let videoElement: HTMLVideoElement;
	let videoPaused = true;
	let videoEnded = false;
	let videoDuration = mediaInfo.duration;
	let videoCurrentTime = 0;
	let videoBuffered: SvelteMediaTimeRange[] = [];
	let videoBuffering = true;
	
	let playerElement: HTMLElement;
	let bottomControlsElement: HTMLElement;
	let topControlsElement: HTMLElement;
	let scrubbingTime: number | null = null;
	let thumbSheetUrl: string | undefined;
	
	let tapBackIndicatorElement: HTMLElement | undefined;
	let tapForwardIndicatorElement: HTMLElement | undefined;
	
	const mobile = window.matchMedia("(pointer: coarse)").matches;
	
	$: videoInfo = mediaInfo.video_info;
	
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
	$: showControls = !isIdle || (!mobile && (bottomControlsElement?.matches(":hover") || topControlsElement?.matches(":hover")));
	
	// Player Actions
	
	function playPause() {
		if (videoPaused || videoEnded) {
			videoElement.play();
			videoPaused = false;
		} else {
			videoElement.pause();
			videoPaused = true;
		}
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
</script>

<svelte:window on:keydown={onWindowKeyPressed}/>

<figure
	class="player-container"
	class:fullscreen={isFullscreen}
	bind:this={playerElement}
	on:pointermove={resetIdleness}
	on:pointerdown={resetIdleness}
	on:fullscreenchange={onFullscreenChange}
>
	<div class="video-container" on:pointerdown={playerClick}>
		{#key mediaInfo.path}
			<VideoElement
				{mediaInfo}
				
				bind:playerBackend
				bind:thumbSheetUrl
				
				bind:videoElement
				bind:videoPaused
				bind:videoEnded
				bind:videoDuration
				bind:videoCurrentTime
				bind:videoBuffered
				bind:videoBuffering
			/>
		{/key}
	</div>
	
	{#if mobile && scrubbingTime !== null && videoInfo !== null && thumbSheetUrl !== undefined}
		<div class="full-thumbnail-container">
			<PreviewThumbnail {videoInfo} {thumbSheetUrl} currentTime={scrubbingTime} extraStyles="flex: 1;"/>
		</div>
	{/if}
	
	{#if videoBuffering}
		<div class="spinner-container">
			<Spinner/>
		</div>
	{/if}
	
	<div bind:this={topControlsElement} class="top-controls hideable" class:hidden={!showControls}>
		<div class="control-row">
			{#if isFullscreen}
				<div class="control-element"><div class="video-title">{mediaInfo.display_name}</div></div>
			{/if}
			
			<div class="spacer"></div>
			
			<ConnectionsButton {mediaInfo} currentTime={videoCurrentTime}/>
		</div>
	</div>
	
	{#if mobile}
		<div class="center-controls hideable" class:hidden={!showControls}>
			<SkipButton floating={true} direction=back {mediaInfo}/>
			<PlayPauseButton floating={true} {videoPaused} on:click={playPause}/>
			<SkipButton floating={true} direction=forward {mediaInfo}/>
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
		{#if mobile}
			<div class="control-row">
				<div class="control-element">{formatDuration(videoCurrentTime)} / {formatDuration(videoDuration)}</div>
				
				<div class="spacer"></div>
				
				<FullscreenButton {isFullscreen} on:click={toggleFullscreen}/>
			</div>
		{/if}
		
		<Timeline
			{mediaInfo}
			{thumbSheetUrl}
			{mobile}
			
			{videoElement}
			bind:videoPaused={videoPaused}
			bind:videoCurrentTime={videoCurrentTime}
			{videoDuration}
			{videoBuffered}
			
			bind:scrubbingTime={scrubbingTime}
		/>
		
		{#if !mobile}
			<div class="control-row">
				<SkipButton direction=back {mediaInfo}/>
				<PlayPauseButton {videoPaused} on:click={playPause}/>
				<SkipButton direction=forward {mediaInfo}/>
				<div class="control-element">{formatDuration(videoCurrentTime)} / {formatDuration(videoDuration)}</div>
				
				<div class="spacer"></div>

				<FullscreenButton {isFullscreen} on:click={toggleFullscreen}/>
			</div>
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
	
	.video-container {
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
	
	.full-thumbnail-container {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
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
		padding-top: 6px;
		padding: 0 8px;
	}
	
	.control-row {
		display: flex;
		align-items: center;
		padding: 8px;
		gap: 8px;
		
		.spacer {
			flex: 1;
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
		position: absolute;
		top: 8px;
		left: 50%;
		transform: translateX(-50%);
		text-align: center;
		font-size: var(--video-player-control-size);
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