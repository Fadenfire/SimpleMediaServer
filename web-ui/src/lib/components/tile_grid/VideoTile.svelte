<script lang="ts">
	import { type Snippet } from "svelte";
    import { escapePath, formatDuration } from "$lib/utils";
    import dayjs from "dayjs";
    import BaseTile from "./BaseTile.svelte";
    import FeatherIcon from "../FeatherIcon.svelte";

	interface Props {
		fileEntry: ApiFileEntry;
		descRow?: Snippet;
		desc?: Snippet;
	}

	let { fileEntry, descRow, desc: descIn }: Props = $props();
	
	let [extraInfo, extraInfoTooltip] = $derived.by(() => {
		const frags = [];
		const creationDate = dayjs(fileEntry.creation_date);
		
		if (fileEntry.artist) frags.push(fileEntry.artist);
		frags.push(creationDate.fromNow());
		
		const extraInfo = frags.join(" • ");
		const extraInfoTooltip = creationDate.format("YYYY-MM-DD");
		
		return [extraInfo, extraInfoTooltip];
	});
</script>

<BaseTile title={fileEntry.display_name} link="/files/{escapePath(fileEntry.full_path)}/" {descRow}>
	{#snippet card()}
		<div class="thumbnail-backdrop">
			<FeatherIcon name="file" size="4em"/>
		</div>
		
		{#key fileEntry.thumbnail_path}
			<img class="thumbnail" src={escapePath(fileEntry.thumbnail_path)} alt="{fileEntry.display_name}">
		{/key}
		
		<div class="duration-container">{formatDuration(fileEntry.duration)}</div>
		
		{#if fileEntry.watch_progress !== null}
			<div class="bar-container">
				<div class="bar" style="width: calc(max(10px, {fileEntry.watch_progress / fileEntry.duration * 100}%));"></div>
			</div>
		{/if}
	{/snippet}
	
	{#snippet desc()}
		<span class="extra-info" title={extraInfoTooltip}>{extraInfo}</span>
		{@render descIn?.()}
	{/snippet}
</BaseTile>

<style lang="scss">
	.thumbnail-backdrop {
		position: absolute;
		left: 0;
		top: 0;
		width: 100%;
		height: 100%;
		z-index: 0;
		display: flex;
		align-items: center;
		justify-content: center;
	}
	
	.thumbnail {
		object-fit: cover;
		width: 100%;
		height: 100%;
		z-index: 1;
	}
	
	.duration-container {
		position: absolute;
		right: 4px;
		bottom: 4px;
		z-index: 2;
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
		z-index: 2;
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
	}
</style>