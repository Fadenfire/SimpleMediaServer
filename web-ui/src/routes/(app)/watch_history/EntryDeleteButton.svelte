<script lang="ts">
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import { createEventDispatcher } from "svelte";

	export let historyEntry: ApiWatchHistoryResponseEntry;
	export let showDeleteButton;
	
	const dispatch = createEventDispatcher();
	
	function deleteEntry() {
		const msg: DeleteWatchProgressParams = {
			library_id: historyEntry.library_id,
			media_path: historyEntry.media_path,
		};
		
		fetch("/api/delete_watch_progress", {
			method: "POST",
			body: JSON.stringify(msg)
		});
		
		dispatch("deleteEntry");
	}
</script>

{#if showDeleteButton}
	<button class="custom-button delete-button" on:click={deleteEntry} title="Delete entry">
		<FeatherIcon name="trash-2" size="1em"/>
	</button>
{/if}

<style lang="scss">
	.delete-button {
		color: #e74c3c;
		font-size: 20px;
	}
</style>
