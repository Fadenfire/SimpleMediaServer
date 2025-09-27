<script lang="ts" module>
	const PAGE_SIZE = 48;
</script>

<script lang="ts">
    import DimStripe from "$lib/components/DimStripe.svelte";
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import PageSection from "$lib/components/PageSection.svelte";
    import SearchBar from "$lib/components/SearchBar.svelte";
    import TileGrid from "$lib/components/tile_grid/TileGrid.svelte";
    import type { PageData } from "./$types";
    import HistoryTile from "./HistoryTile.svelte";
	
	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();
	
	let deletedEntries: ApiWatchHistoryEntry[] = $state([]);
	let searchText = $state("");
	
	let historyPromise: Promise<ApiWatchHistoryResponse> = $derived.by(async () => {
		const searchQuery = searchText
			.trim()
			.toLowerCase();
		
		let url = `/api/watch_history?page=${data.pageIndex}&page_size=${PAGE_SIZE}`;
		
		if (searchQuery !== "") {
			url += `&search_query=${encodeURIComponent(searchQuery)}`;
		}
		
		const res = await fetch(url);
        return await res.json();
	});
</script>

<svelte:head>
	<title>Watch History - Media Server</title>
</svelte:head>

<main class="main-content">
	<PageSection title="Watch History">
		{#snippet titleBar()}
			<SearchBar onCommited={(contents) => searchText = contents} placeholder="Filter by name"/>
		{/snippet}
		
		{#await historyPromise}
			<DimStripe>Loading</DimStripe>
		{:then watchHistory}
			<TileGrid>
				{#each watchHistory.entries as entry}
					{#if !deletedEntries.some(e => e.media_path === entry.media_path && e.library_id === entry.library_id)}
						<HistoryTile
							historyEntry={entry}
							showDeleteButton={true}
							showLastWatched={true}
							deleteEntry={() => deletedEntries = [...deletedEntries, entry]}
						/>
					{/if}
				{/each}
			</TileGrid>
			
			<div class="footer">
				<a
					class="page-link"
					href={data.pageIndex > 0 ? `?page=${data.pageIndex - 1}` : undefined}
					title="Previous page"
				>
					<FeatherIcon name="arrow-left" size="1em"/>
				</a>
				
				<span>{data.pageIndex + 1} of {watchHistory.total_pages}</span>
				
				<a
					class="page-link"
					href={data.pageIndex < watchHistory.total_pages - 1 ? `?page=${data.pageIndex + 1}` : undefined}
					title="Next page"
				>
					<FeatherIcon name="arrow-right" size="1em"/>
				</a>
			</div>
		{/await}
	</PageSection>
</main>

<style lang="scss">
	.footer {
		padding: 8px;
		margin-top: 48px;
		display: flex;
		flex-direction: row;
		gap: 8px;
		align-items: center;
		justify-content: center;
		
		.page-link {
			text-decoration: none;
			color: var(--main-text-color);
			font-size: 24px;
			
			&:not([href]) {
				color: var(--disabled-text-color);
			}
		}
	}
</style>