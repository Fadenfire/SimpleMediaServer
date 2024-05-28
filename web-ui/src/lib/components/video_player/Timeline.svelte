<script lang="ts" context="module">
	export function caclulateThumbnailSheetOffset(time: number, videoInfo: VideoInfo) {
		const offset = Math.floor(time / videoInfo.thumbnail_sheet_interval);
		
		return {
			spriteX: Math.floor(offset % videoInfo.thumbnail_sheet_cols),
			spriteY: Math.floor(offset / videoInfo.thumbnail_sheet_rows)
		}
	}
</script>

<script lang="ts">
    import type { SvelteMediaTimeRange } from "svelte/elements";
    import Bar from "./Bar.svelte";
    import { formatDuration } from "$lib/utils";
	
	export let mediaInfo: MediaInfo;
	export let thumbSheetUrl: string | undefined;
	export let mobile: boolean;
	
	export let videoElement: HTMLVideoElement;
	export let videoCurrentTime: number;
	export let videoDuration: number;
	export let videoBuffered: SvelteMediaTimeRange[];
	
	export let scrubbingTime: number | null = null;
	
	let timelineElement: HTMLElement;
	
	$: videoInfo = mediaInfo.video_info;
	
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
			wasPaused = videoElement.paused || videoElement.ended;
			
			if (!wasPaused) videoElement.pause();
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
			
			if (!wasPaused) videoElement.play();
		}
	}
	
	function normalizePointerPos(pointerX: number): number {
		const rect = timelineElement.getBoundingClientRect();
		
		return Math.min(1.0, Math.max(0.0, (pointerX - rect.x) / rect.width));
	}
</script>

<svelte:window on:pointermove={onWindowPointerMove} on:pointerup={onWindowPointerUp} />

<div class="timeline" class:expanded={mobile} bind:this={timelineElement}>
	<div class="bounding-box" class:mobile={mobile} class:scrubbing={scrubbingTime !== null} on:pointerdown={onTimelinePointerDown} on:pointermove={onTimelinePointerMove} on:pointerleave={onTimelinePointerLeave}>
		<div class="bars">
			<Bar color="var(--video-player-base-bar-color)"/>
			
			{#each videoBuffered as seg}
				<Bar color="var(--video-player-buffer-bar-color)" startValue={seg.start / videoDuration} endValue={seg.end / videoDuration} />
			{/each}
			
			{#if !mobile && hoverProgress !== null}
				<Bar color="var(--video-player-buffer-bar-color)" endValue={hoverProgress} />
			{/if}
			
			<Bar color="var(--video-player-accent-bar-color)" endValue={videoCurrentTime / videoDuration} />
			
			<div class="thumb-wrapper" style="transform: translateX({videoCurrentTime / videoDuration * 100}cqw);">
				<div class="thumb"></div>
			</div>
			
			{#if !mobile && hoverProgress !== null}
				<div class="tooltip" style="transform: translateX({(hoverProgress) * 100}cqw) translate(-50%, -12px);">
					{#if videoInfo !== null && thumbSheetUrl !== undefined}
						{@const thumbOffset = caclulateThumbnailSheetOffset(hoverProgress * videoDuration, videoInfo)}
						<div
							class="timeline-thumbnail"
							style="
								background-image: url({thumbSheetUrl});
								background-position: -{thumbOffset.spriteX * 100}% -{thumbOffset.spriteY * 100}%;
								background-size: {videoInfo.thumbnail_sheet_cols * 100}% {videoInfo.thumbnail_sheet_rows * 100}%;
								aspect-ratio: {videoInfo.sheet_thumbnail_size.width} / {videoInfo.sheet_thumbnail_size.height};
							"
						></div>
					{/if}
					
					{formatDuration((hoverProgress) * videoDuration)}
				</div>
			{/if}
		</div>
	</div>
</div>

<style lang="scss">
	@mixin no-select {
		user-select: none;
		-webkit-user-select: none;
	}
	
	.timeline {
		position: relative;
		width: 100%;
		height: var(--video-player-bar-width);
		overflow: visible;
		@include no-select;
		
		&.expanded {
			margin-top: 4px;
			margin-bottom: 16px;
		}
	}
	
	.bounding-box {
		position: absolute;
		top: 50%;
		left: 0;
		transform: translateY(-50%);
		display: flex;
		align-items: center;
		width: 100%;
		height: var(--video-player-timeline-width);
		cursor: pointer;
		touch-action: none;
		@include no-select;
	}
	
	.bars {
		position: relative;
		width: 100%;
		height: var(--video-player-bar-width);
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
		width: var(--video-player-thumb-size);
		height: var(--video-player-thumb-size);
		transform: translate(-50%, -50%) scale(1);
		border-radius: 50%;
		background-color: var(--video-player-accent-bar-color);
		transition: transform 0.2s;
		touch-action: none;
		@include no-select;
	}
	
	.tooltip {
		position: absolute;
		bottom: 0;
		left: 0;
		font-size: 10px;
		font-weight: 500;
		text-align: center;
		background-color: #000C;
		padding: 4px;
	}
	
	.timeline-thumbnail {
		height: 72px;
	}
	
	.bounding-box:not(.mobile):hover, .bounding-box.scrubbing {
		.bars {
			height: var(--video-player-bar-focused-width);
		}
	}
	
	.bounding-box:not(.mobile, :hover, .scrubbing) .thumb {
		transform: translate(-50%, -50%) scale(0);
	}
</style>