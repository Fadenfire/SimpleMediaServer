<script lang="ts">
    import DimStripe from "$lib/components/DimStripe.svelte";
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import PageSection from "$lib/components/PageSection.svelte";
    import TileGrid from "$lib/components/tile_grid/TileGrid.svelte";
    import type { PageData } from "./$types";
    import HistoryTile from "./HistoryTile.svelte";
	
	export let data: PageData;
</script>

<svelte:head>
	<title>Watch History - Media Server</title>
</svelte:head>

<main class="main-content">
	<PageSection title="Watch History">
		{#await data.historyPromise}
			<DimStripe>Loading</DimStripe>
		{:then watchHistory}
			<TileGrid>
				{#each watchHistory.entries as entry}
					<HistoryTile
						historyEntry={entry}
						showDeleteButton={true}
						on:deleteEntry={() => watchHistory.entries = watchHistory.entries.filter(e => e !== entry)}
					/>
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