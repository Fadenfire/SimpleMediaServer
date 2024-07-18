<script lang="ts">
    import type { SvelteMediaTimeRange } from "svelte/elements";
    import Bar from "./Bar.svelte";
    import { formatDuration } from "$lib/utils";
    import PreviewThumbnail from "./PreviewThumbnail.svelte";
    import { isMobile } from "./VideoPlayer.svelte";
	
	export let mediaInfo: ApiFileInfo;
	export let thumbSheetUrl: string | undefined;
	
	export let videoElement: HTMLVideoElement;
	export let videoPaused: boolean;
	export let videoCurrentTime: number;
	export let videoDuration: number;
	export let videoBuffered: SvelteMediaTimeRange[];
	
	export let scrubbingTime: number | null = null;
	
	let timelineElement: HTMLElement;
	
	$: videoInfo = mediaInfo.video_info;
	
	const mobile = isMobile();
	
	// Hover Effects
	
	let pureHoverProgress: number | null = null;
	
	function onTimelinePointerMove(event: PointerEvent) {
		if (!mobile) {
			pureHoverProgress = normalizePointerPos(event.clientX);
		}
	}
	
	function onTimelinePointerLeave() {
		pureHoverProgress = null;
	}
	
	$: hoverProgress = scrubbingTime !== null ? scrubbingTime / videoDuration : pureHoverProgress;
	
	// Scrubbing
	
	let wasPaused = true;
	
	function updateScrub(pointerX: number) {
		scrubbingTime = normalizePointerPos(pointerX) * videoDuration;
		videoCurrentTime = scrubbingTime;
	}
	
	function onTimelinePointerDown(event: PointerEvent) {
		if (event.button == 0) {
			wasPaused = videoPaused;
			videoElement.pause();
			
			updateScrub(event.clientX);
			
			event.preventDefault();
		}
	}
	
	function onWindowPointerMove(event: PointerEvent) {
		if (scrubbingTime !== null) {
			updateScrub(event.clientX);
		}
	}
	
	function onWindowPointerUp() {
		if (scrubbingTime !== null) {
			videoElement.currentTime = scrubbingTime;
			scrubbingTime = null;
			
			if (wasPaused) {
				videoElement.pause();
			} else {
				videoElement.play();
			}
		}
	}
	
	function normalizePointerPos(pointerX: number): number {
		const rect = timelineElement.getBoundingClientRect();
		
		return Math.min(1.0, Math.max(0.0, (pointerX - rect.x) / rect.width));
	}
</script>

<svelte:window on:pointermove={onWindowPointerMove} on:pointerup={onWindowPointerUp} />

<div class="timeline" bind:this={timelineElement}>
	<div
		class="bounding-box"
		class:mobile={mobile}
		class:scrubbing={scrubbingTime !== null}
		on:pointerdown={onTimelinePointerDown}
		on:pointermove={onTimelinePointerMove}
		on:pointerleave={onTimelinePointerLeave}
	>
		<div class="bars">
			<Bar color="var(--background-bar-color)"/>
			
			{#each videoBuffered as seg}
				<Bar color="var(--foreground-bar-color)" startValue={seg.start / videoDuration} endValue={seg.end / videoDuration} />
			{/each}
			
			{#if !mobile && hoverProgress !== null}
				<Bar color="var(--foreground-bar-color)" endValue={hoverProgress} />
			{/if}
			
			<Bar color="var(--accent-bar-color)" endValue={videoCurrentTime / videoDuration} />
			
			<div class="thumb-wrapper" style="transform: translateX({videoCurrentTime / videoDuration * 100}cqw);">
				<div class="thumb"></div>
			</div>
			
			{#if !mobile && hoverProgress !== null}
				<div class="tooltip" style="transform: translateX({hoverProgress * 100}cqw) translate(-50%, -12px);">
					{#if videoInfo !== null && thumbSheetUrl !== undefined}
						<PreviewThumbnail
							{videoInfo}
							{thumbSheetUrl}
							currentTime={hoverProgress * videoDuration}
							extraStyles="height: 92px;"
						/>
					{/if}
					
					{formatDuration((hoverProgress) * videoDuration)}
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