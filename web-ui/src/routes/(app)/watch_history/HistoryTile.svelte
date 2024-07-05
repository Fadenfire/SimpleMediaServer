<script lang="ts">
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import BaseTile from "$lib/components/tile_grid/BaseTile.svelte";
	import VideoTile from "$lib/components/tile_grid/VideoTile.svelte";
    import { escapePath } from "$lib/utils";
    import EntryDeleteButton from "./EntryDeleteButton.svelte";
	
	export let historyEntry: ApiWatchHistoryEntry;
	export let showDeleteButton = false;
</script>

{#if historyEntry.file !== null}
	<VideoTile fileEntry={historyEntry.file}>
		<EntryDeleteButton slot="title-row" {historyEntry} {showDeleteButton} on:deleteEntry/>
	</VideoTile>
{:else}
	<BaseTile title="Removed Video">
		<FeatherIcon slot="card" name="file" size="4em"/>
		<EntryDeleteButton slot="title-row" {historyEntry} {showDeleteButton} on:deleteEntry/>
	</BaseTile>
{/if}
