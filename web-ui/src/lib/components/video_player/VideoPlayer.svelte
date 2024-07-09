<script lang="ts" context="module">
	export const TAP_SEEK_AMOUNT = 10;

	export function isMobile() {
		// return true;
		return window.matchMedia("(pointer: coarse)").matches;
	}
	
	export enum SidebarType {
		None,
		Connections,
		Settings,
	}
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
    import ConnectionsMenu from "./menus/ConnectionsMenu.svelte";
    import SettingsButton from "./buttons/SettingsButton.svelte";
	import SettingsMenu from "./menus/SettingsMenu.svelte";

	export let mediaInfo: ApiFileInfo;
	
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
	let scrubbingTime: number | null = null;
	let thumbSheetUrl: string | undefined;
	
	let tapBackIndicatorElement: HTMLElement | undefined;
	let tapForwardIndicatorElement: HTMLElement | undefined;
	
	const mobile = isMobile();
	
	$: videoInfo = mediaInfo.video_info;
	
	// Sidedar
	
	let sidebarShown = SidebarType.None;
	
	function toggleSidebar(sidebar: SidebarType) {
		sidebarShown = sidebarShown == sidebar ? SidebarType.None : sidebar;
	}
	
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
	
	let controlsContainerElement: HTMLElement;
	
	let showControls = false;
	$: showControls = !isIdle || sidebarShown != SidebarType.None || (!mobile && controlsContainerElement?.matches(":hover"));
	
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
	class="video-player"
	class:fullscreen={isFullscreen}
	class:mobile
	bind:this={playerElement}
	on:pointermove={resetIdleness}
	on:pointerdown={resetIdleness}
	on:fullscreenchange={onFullscreenChange}
>
	<div class="video-container" on:pointerdown={playerClick}>
		{#key mediaInfo.full_path}
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
	
	{#if scrubbingTime !== null && videoInfo !== null && thumbSheetUrl !== undefined}
		<div class="full-thumbnail-container">
			<PreviewThumbnail {videoInfo} {thumbSheetUrl} currentTime={scrubbingTime} extraStyles="flex: 1;"/>
		</div>
	{/if}
	
	{#if videoBuffering}
		<div class="spinner-container">
			<Spinner/>
		</div>
	{/if}
	
	{#if mobile}
		<div class="center-controls hideable" class:hidden={!showControls}>
			<SkipButton floating={true} direction=back {mediaInfo}/>
			<PlayPauseButton floating={true} {videoPaused} on:click={playPause}/>
			<SkipButton floating={true} direction=forward {mediaInfo}/>
		</div>
		
		{#if tapSeekBackAmount > 0}
			<div class="tap-seek-indicator left" bind:this={tapBackIndicatorElement}>
				<FeatherIcon name="rewind" size="3em"/>
				{tapSeekBackAmount} seconds
			</div>
		{/if}
		
		{#if tapSeekForwardAmount > 0}
			<div class="tap-seek-indicator right" bind:this={tapForwardIndicatorElement}>
				<FeatherIcon name="fast-forward" size="3em"/>
				{tapSeekForwardAmount} seconds
			</div>
		{/if}
	{/if}
	
	<div bind:this={controlsContainerElement} class="controls-container hideable" class:hidden={!showControls}>
		<div class="top-controls">
			<div class="control-row">
				{#if isFullscreen}
					<div class="control-element"><div class="video-title">{mediaInfo.display_name}</div></div>
				{/if}
				
				<div class="flex-spacer"></div>
				
				<ConnectionsButton {mediaInfo} {videoCurrentTime} on:click={() => toggleSidebar(SidebarType.Connections)}/>
			</div>
		</div>
		
		<div class="bottom-controls">
			{#if mobile}
				<div class="control-row">
					<div class="control-element">{formatDuration(videoCurrentTime)} / {formatDuration(videoDuration)}</div>
					
					<div class="flex-spacer"></div>
					
					<SettingsButton on:click={() => toggleSidebar(SidebarType.Settings)}/>
					<FullscreenButton {isFullscreen} on:click={toggleFullscreen}/>
				</div>
			{/if}
			
			<div class:mobile-timeline-container={mobile} class:fullscreen={isFullscreen}>
				<Timeline
					{mediaInfo}
					{thumbSheetUrl}
					
					{videoElement}
					bind:videoPaused={videoPaused}
					bind:videoCurrentTime={videoCurrentTime}
					{videoDuration}
					{videoBuffered}
					
					bind:scrubbingTime={scrubbingTime}
				/>
			</div>
			
			{#if !mobile}
				<div class="control-row">
					<SkipButton direction=back {mediaInfo}/>
					<PlayPauseButton {videoPaused} on:click={playPause}/>
					<SkipButton direction=forward {mediaInfo}/>
					<div class="control-element">{formatDuration(videoCurrentTime)} / {formatDuration(videoDuration)}</div>
					
					<div class="flex-spacer"></div>
					
					<SettingsButton on:click={() => toggleSidebar(SidebarType.Settings)}/>
					<FullscreenButton {isFullscreen} on:click={toggleFullscreen}/>
				</div>
			{/if}
		</div>
		
		<div class="right-sidebar">
			{#if sidebarShown == SidebarType.Connections}
				<ConnectionsMenu {mediaInfo} {videoElement} {videoCurrentTime}/>
			{/if}
			
			<div class="flex-spacer"></div>
			
			{#if sidebarShown == SidebarType.Settings && playerBackend}
				<SettingsMenu {playerBackend}/>
			{/if}
		</div>
	</div>
</figure>

<style lang="scss">
	@use "player.scss";
	
	$seek-indicator-size: 90px;
	
	.video-player {
		width: 100%;
		height: 100%;
		background-color: black;
		position: relative;
		touch-action: manipulation;
		
		--video-player-control-gap: 8px;
		--video-player-control-size: 20px;
		--video-player-large-control-size: 30px;
		
		&.fullscreen {
			touch-action: none;
		}
		
		&.mobile {
			--video-player-control-gap: 12px;
			--video-player-control-size: 24px;
			--video-player-large-control-size: 34px;
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
	
	.controls-container {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		display: grid;
		grid-template-areas:
			"top top top"
			". . right"
			"bottom bottom bottom";
		grid-template-columns: auto 1fr auto;
		grid-template-rows: auto 1fr auto;
		pointer-events: none;
		
		> * {
			pointer-events: auto;
		}
	}
	
	.flex-spacer {
		flex: 1;
		pointer-events: none;
	}
	
	.top-controls {
		grid-area: top;
		position: relative;
		background: linear-gradient(to bottom, rgba(black, 0.6), transparent);
		padding: 0 player.$gap-size;
		
		.control-row {
			box-sizing: content-box;
			height: var(--video-player-large-control-size);
		}
	}
	
	.bottom-controls {
		grid-area: bottom;
		display: flex;
		flex-direction: column;
		background: linear-gradient(to top, rgba(black, 0.6), transparent);
		padding: 0 player.$gap-size;
		padding-top: player.$gap-size;
	}
	
	.right-sidebar {
		grid-area: right;
		display: flex;
		flex-direction: column;
		justify-content: flex-start;
		padding: player.$gap-size;
		overflow: hidden;
		pointer-events: none;
		
		> :global(*) {
			pointer-events: auto;
		}
	}
	
	.mobile-timeline-container {
		padding-top: 4px;
		padding-bottom: 16px;
		
		&.fullscreen {
			padding-bottom: 32px;
		}
	}
	
	.control-row {
		display: flex;
		align-items: center;
		padding: player.$gap-size;
		gap: var(--video-player-control-gap);
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
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		text-align: center;
		font-size: var(--video-player-control-size);
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
		@include player.floating-circle;
		@include player.floating-circle-large;
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
		@include player.floating-circle;
		flex-direction: column;
		width: $seek-indicator-size;
		height: $seek-indicator-size;
		font-size: 12px;
		font-weight: 500;
		animation: fadeOut 1s 0.5s forwards;
		pointer-events: none;
		
		&.left {
			left: $seek-indicator-size;
		}
		
		&.right {
			right: $seek-indicator-size;
		}
	}
</style>