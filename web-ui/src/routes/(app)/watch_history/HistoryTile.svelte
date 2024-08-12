<script lang="ts">
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import BaseTile from "$lib/components/tile_grid/BaseTile.svelte";
	import VideoTile from "$lib/components/tile_grid/VideoTile.svelte";
    import dayjs from "dayjs";
    import EntryDeleteButton from "./EntryDeleteButton.svelte";
	
	export let historyEntry: ApiWatchHistoryEntry;
	export let showDeleteButton = false;
	export let showLastWatched = false;
	
	$: lastWatched = showLastWatched ? dayjs(historyEntry.last_watched).fromNow() : undefined;
</script>

{#if historyEntry.file !== null}
	<VideoTile fileEntry={historyEntry.file}>
		<EntryDeleteButton slot="desc-row" {historyEntry} {showDeleteButton} on:deleteEntry/>
		
		<svelte:fragment slot="desc">
			{#if lastWatched}
				<span class="extra-info">Last watched {lastWatched}</span>
			{/if}
		</svelte:fragment>
	</VideoTile>
{:else}
	<BaseTile title="Removed Video">
		<FeatherIcon slot="card" name="file" size="4em"/>
		<EntryDeleteButton slot="desc-row" {historyEntry} {showDeleteButton} on:deleteEntry/>
		
		<svelte:fragment slot="desc">
			{#if lastWatched}
				<span class="extra-info">Last watched {lastWatched}</span>
			{/if}
		</svelte:fragment>
	</BaseTile>
{/if}

<style lang="scss">
	.extra-info {
		color: var(--secondary-text-color);
		font-size: 0.8em;
	}
</style>