<script lang="ts">
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";

	interface Props {
		historyEntry: ApiWatchHistoryEntry;
		showDeleteButton: boolean;
		deleteEntry?: () => void;
	}

	let {
		historyEntry,
		showDeleteButton,
		deleteEntry: deleteEntryCallback
	}: Props = $props();
	
	function deleteEntry() {
		const msg: DeleteWatchProgressParams = {
			library_id: historyEntry.library_id,
			media_path: historyEntry.media_path,
		};
		
		fetch("/api/delete_watch_progress", {
			method: "POST",
			body: JSON.stringify(msg)
		});
		
		deleteEntryCallback?.();
	}
</script>

{#if showDeleteButton}
	<button class="custom-button delete-button" onclick={() => deleteEntry()} title="Delete entry">
		<FeatherIcon name="trash-2" size="1em"/>
	</button>
{/if}

<style lang="scss">
	.delete-button {
		color: #e74c3c;
		font-size: 20px;
		align-self: flex-start;
	}
</style>
