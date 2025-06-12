<script lang="ts" module>
	export const PRECISE_SCRUBBING_START_DIST = 20;
	export const PRECISE_SCRUBBING_SPEEDS = [60, 30, 10, 5, 1];
</script>

<script lang="ts">
    import Bar from "./Bar.svelte";
    import { formatDuration } from "$lib/utils";
    import PreviewThumbnail from "./PreviewThumbnail.svelte";
    import { isMobile } from "./VideoPlayer.svelte";
    import type { VideoElementState } from "./VideoElement.svelte";
	
	interface Props {
		mediaInfo: ApiFileInfo;
		videoState: VideoElementState;
		playerElement: HTMLElement | undefined;
		scrubbingTime?: number | null;
		preciseScrubbing?: boolean;
	}

	let {
		mediaInfo,
		videoState,
		playerElement,
		scrubbingTime = $bindable(null),
		preciseScrubbing = $bindable(false)
	}: Props = $props();
	
	let timelineElement: HTMLElement | undefined = $state();
	
	let videoInfo = $derived(mediaInfo.video_info);
	
	const mobile = isMobile();
	
	// Hover Effects
	
	let pureHoverProgress: number | null = $state(null);
	
	function onTimelinePointerMove(event: PointerEvent) {
		if (!mobile) {
			pureHoverProgress = normalizePointerPos(event.clientX);
		}
	}
	
	function onTimelinePointerLeave() {
		pureHoverProgress = null;
	}
	
	let hoverProgress = $derived(scrubbingTime !== null ? scrubbingTime / videoState.duration : pureHoverProgress);
	
	// Scrubbing
	
	let wasPaused = true;
	let scrubSpeed = $state(0);
	let lastTickScrubPos: number | null = null;
	
	function updateScrub(pointerX: number, pointerY: number) {
		if (timelineElement === undefined) return;
		if (playerElement === undefined) return;
		if (videoState.videoElement === undefined) return;
		
		const normPos = normalizePointerPos(pointerX);
		
		if (lastTickScrubPos === null) {
			scrubbingTime = normPos * videoState.duration;
			preciseScrubbing = false;
		} else {
			const distAbove = timelineElement.getBoundingClientRect().y - pointerY;
			
			if (distAbove > PRECISE_SCRUBBING_START_DIST) {
				const playerHeight = playerElement.getBoundingClientRect().height;
				
				const amount = (distAbove - PRECISE_SCRUBBING_START_DIST) / (playerHeight * 0.8);
				const index = Math.min(PRECISE_SCRUBBING_SPEEDS.length - 1, Math.max(0.0, Math.floor(amount * PRECISE_SCRUBBING_SPEEDS.length)));
				
				scrubSpeed = PRECISE_SCRUBBING_SPEEDS[index];
				preciseScrubbing = true;
			} else {
				scrubSpeed = videoState.duration;
				preciseScrubbing = false;
			}
			
			scrubbingTime = (scrubbingTime ?? normPos) + (normPos - lastTickScrubPos) * scrubSpeed;
		}
		
		videoState.currentTime = scrubbingTime;
		if (preciseScrubbing) videoState.videoElement.currentTime = scrubbingTime;
		
		lastTickScrubPos = normPos;
	}
	
	function onTimelinePointerDown(event: PointerEvent) {
		if (event.button == 0) {
			if (videoState.videoElement === undefined) return;
			
			wasPaused = videoState.isPaused;
			videoState.videoElement.pause();
			
			updateScrub(event.clientX, event.clientY);
			
			event.preventDefault();
		}
	}
	
	function onWindowPointerMove(event: PointerEvent) {
		if (scrubbingTime !== null) {
			updateScrub(event.clientX, event.clientY);
		}
	}
	
	function onWindowPointerUp() {
		if (videoState.videoElement === undefined) return;
		
		if (scrubbingTime !== null) {
			videoState.videoElement.currentTime = scrubbingTime;
			scrubbingTime = null;
			lastTickScrubPos = null;
			preciseScrubbing = false;
			
			if (wasPaused) {
				videoState.videoElement.pause();
			} else {
				videoState.videoElement.play();
			}
		}
	}
	
	function normalizePointerPos(pointerX: number): number {
		if (timelineElement === undefined) return 0.0;
		
		const rect = timelineElement.getBoundingClientRect();
		
		return Math.min(1.0, Math.max(0.0, (pointerX - rect.x) / rect.width));
	}
</script>

<svelte:window onpointermove={onWindowPointerMove} onpointerup={onWindowPointerUp} />

<div class="timeline" bind:this={timelineElement}>
	<div
		class="bounding-box"
		class:mobile={mobile}
		class:scrubbing={scrubbingTime !== null}
		onpointerdown={onTimelinePointerDown}
		onpointermove={onTimelinePointerMove}
		onpointerleave={onTimelinePointerLeave}
	>
		<div class="bars">
			<Bar color="var(--background-bar-color)"/>
			
			{#each videoState.buffered as seg}
				<Bar color="var(--foreground-bar-color)" startValue={seg.start / videoState.duration} endValue={seg.end / videoState.duration} />
			{/each}
			
			{#if !mobile && hoverProgress !== null}
				<Bar color="var(--foreground-bar-color)" endValue={hoverProgress} />
			{/if}
			
			<Bar color="var(--accent-bar-color)" endValue={videoState.currentTime / videoState.duration} />
			
			<div class="thumb-wrapper" style="transform: translateX({videoState.currentTime / videoState.duration * 100}cqw);">
				<div class="thumb"></div>
			</div>
			
			{#if (!mobile || preciseScrubbing) && hoverProgress !== null}
				<div class="tooltip" style="transform: translateX({hoverProgress * 100}cqw) translate(-50%, -12px);">
					{#if !mobile}
						{#if videoInfo !== null && videoState.thumbSheetUrl !== undefined}
							<PreviewThumbnail
								{videoInfo}
								thumbSheetUrl={videoState.thumbSheetUrl}
								currentTime={hoverProgress * videoState.duration}
								extraStyles="height: 92px;"
							/>
						{/if}
						
						<span>{formatDuration((hoverProgress) * videoState.duration)}</span>
					{/if}
					
					{#if preciseScrubbing}
						<span>Precise Scrubbing: {scrubSpeed}s</span>
					{/if}
				</div>
			{/if}
		</div>
	</div>
</div>

<style lang="scss">
	$timeline-bounding-height: 14px;
	$timeline-bounding-mobile-height: 24px;
	$timeline-bar-focused-width: 5px;
	$thumb-size: 10px;
	
	@mixin no-select {
		user-select: none;
		-webkit-user-select: none;
	}
	
	.timeline {
		--background-bar-color: #DDD6;
		--foreground-bar-color: #EEE6;
		
		position: relative;
		width: 100%;
		height: var(--bar-width);
		overflow: visible;
		@include no-select;
	}
	
	.bounding-box {
		position: absolute;
		top: 50%;
		left: 0;
		transform: translateY(-50%);
		display: flex;
		align-items: center;
		width: 100%;
		height: $timeline-bounding-height;
		cursor: pointer;
		touch-action: none;
		@include no-select;
		
		&.mobile {
			height: $timeline-bounding-mobile-height;
		}
	}
	
	.bars {
		position: relative;
		width: 100%;
		height: var(--bar-width);
		overflow: visible;
		container-type: size;
		transition: height 0.2s;
		@include no-select;
	}
	
	.thumb-wrapper {
		position: absolute;
		top: 50%;
		left: 0;
		@include no-select;
	}
	
	.thumb {
		width: $thumb-size;
		height: $thumb-size;
		transform: translate(-50%, -50%) scale(1);
		border-radius: 50%;
		background-color: var(--accent-bar-color);
		transition: transform 0.2s;
		touch-action: none;
		@include no-select;
	}
	
	.tooltip {
		position: absolute;
		bottom: 0;
		left: 0;
		z-index: 100;
		display: flex;
		flex-direction: column;
		align-items: center;
		font-size: 10px;
		font-weight: 500;
		text-align: center;
		background-color: #000C;
		border-radius: 4px;
		padding: 4px;
	}
	
	.bounding-box:not(.mobile):hover, .bounding-box.scrubbing {
		.bars {
			height: $timeline-bar-focused-width;
		}
	}
	
	.bounding-box:not(.mobile, :hover, .scrubbing) .thumb {
		transform: translate(-50%, -50%) scale(0);
	}
</style>