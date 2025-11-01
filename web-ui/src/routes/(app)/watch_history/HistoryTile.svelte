<script lang="ts">
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import BaseTile from "$lib/components/tile_grid/BaseTile.svelte";
	import VideoTile from "$lib/components/tile_grid/VideoTile.svelte";
    import dayjs from "dayjs";
    import EntryDeleteButton from "./EntryDeleteButton.svelte";
	
	interface Props {
		historyEntry: ApiWatchHistoryEntry;
		showDeleteButton?: boolean;
		showLastWatched?: boolean;
		deleteEntry?: () => void;
	}

	let {
		historyEntry,
		showDeleteButton = false,
		showLastWatched = false,
		deleteEntry
	}: Props = $props();
	
	let lastWatched = $derived(dayjs(historyEntry.last_watched));
	let lastWatchedText = $derived(lastWatched.fromNow());
	let lastWatchedTooltip = $derived(lastWatched.format("YYYY-MM-DD hh:mm A"));
</script>

{#if historyEntry.file !== null}
	<VideoTile fileEntry={historyEntry.file}>
		{#snippet descRow()}
			<EntryDeleteButton {historyEntry} {showDeleteButton} {deleteEntry}/>
		{/snippet}
		
		{#snippet desc()}
			{#if showLastWatched}
				<span class="extra-info" title={lastWatchedTooltip}>Last watched {lastWatchedText}</span>
			{/if}
		{/snippet}
	</VideoTile>
{:else}
	<BaseTile title="Removed Video">
		{#snippet card()}
			<FeatherIcon  name="file" size="4em"/>
		{/snippet}
		
		{#snippet descRow()}
			<EntryDeleteButton {historyEntry} {showDeleteButton} {deleteEntry}/>
		{/snippet}
		
		{#snippet desc()}
			{#if showLastWatched}
				<span class="extra-info" title={lastWatchedTooltip}>Last watched {lastWatchedText}</span>
			{/if}
		{/snippet}
	</BaseTile>
{/if}

<style lang="scss">
	.extra-info {
		color: var(--secondary-text-color);
		font-size: 0.8em;
	}
</style>