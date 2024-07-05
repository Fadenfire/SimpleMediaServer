<script lang="ts">
    import { escapePath, formatDuration } from "$lib/utils";
    import BaseTile from "./BaseTile.svelte";

	export let fileEntry: ApiFileEntry;
</script>

<BaseTile title={fileEntry.display_name} link="/files/{escapePath(fileEntry.full_path)}/">
	<svelte:fragment slot="card">
		<img class="thumbnail" src={escapePath(fileEntry.thumbnail_path)} alt="{fileEntry.display_name}">
		<div class="duration-container">{formatDuration(fileEntry.duration)}</div>
		
		{#if fileEntry.watch_progress !== null}
			<div class="bar-container">
				<div class="bar" style="width: calc(max(10px, {fileEntry.watch_progress / fileEntry.duration * 100}%));"></div>
			</div>
		{/if}
	</svelte:fragment>
	
	<slot name="title-row" slot="title-row"></slot>
	
	<svelte:fragment slot="desc">
		{#if fileEntry.artist}
			<span class="extra-info">{fileEntry.artist}</span>
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
		height: var(--bar-width);
		background-color: #DDD9;
		
		.bar {
			height: 100%;
			background-color: var(--accent-bar-color);
		}
	}
	
	.extra-info {
		color: var(--secondary-text-color);
		font-size: 0.8em;
		line-height: 2em;
	}
</style>