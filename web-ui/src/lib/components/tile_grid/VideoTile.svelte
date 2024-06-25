<script lang="ts">
    import { formatDuration } from "$lib/utils";
    import BaseTile from "./BaseTile.svelte";

	export let title: string;
	export let duration: number;
	export let link: string;
	export let thumbnailPath: string;
	export let progress: number | null = null;
</script>

<BaseTile title="{title}" link="{link}">
	<svelte:fragment slot="card">
		<img class="thumbnail" src="{thumbnailPath}" alt="{title}">
		<div class="duration-container">{formatDuration(duration)}</div>
		
		{#if progress !== null}
			<div class="bar-container">
				<div class="bar" style="width: calc(max(10px, {progress / duration * 100}%));"></div>
			</div>
		{/if}
	</svelte:fragment>
</BaseTile>

<style lang="scss">
	.thumbnail {
		object-fit: cover;
		width: 100%;
		height: 100%;
	}
	
	.duration-container {
		position: absolute;
		right: 4px;
		bottom: 4px;
		background-color: rgba(#000, 0.7);
		font-size: 12px;
		padding: 3px 5px;
		border-radius: 4px;
		font-weight: 500;
	}
	
	.bar-container {
		position: absolute;
		left: 0px;
		bottom: 0px;
		width: 100%;
		height: 3px;
		background-color: #DDD9;
		
		.bar {
			height: 100%;
			background-color: var(--video-player-accent-bar-color);
		}
	}
</style>