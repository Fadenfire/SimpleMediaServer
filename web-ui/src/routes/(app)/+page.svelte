<script lang="ts">
    import DirectoryTile from "$lib/components/tile_grid/DirectoryTile.svelte";
	import PageSection from "$lib/components/PageSection.svelte";
	import TileGrid from "$lib/components/tile_grid/TileGrid.svelte";
	import type { PageData } from "./$types";
    import HistoryTile from "./watch_history/HistoryTile.svelte";

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();
</script>

<svelte:head>
	<title>Home - Media Server</title>
</svelte:head>

<main class="main-content">
	<PageSection title="Libraries">
		<TileGrid>
			{#each data.libraries as library}
				<DirectoryTile title={library.display_name} link="/files/{encodeURIComponent(library.id)}/" />
			{/each}
		</TileGrid>
	</PageSection>
	
	<PageSection title="Recently Watched" titleLink="/watch_history">
		<TileGrid>
			{#each data.watchHistory.entries as entry}
				<HistoryTile historyEntry={entry}/>
			{/each}
		</TileGrid>
	</PageSection>
</main>
