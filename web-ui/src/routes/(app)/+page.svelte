<script lang="ts">
    import DirectoryTile from "$lib/components/tile_grid/DirectoryTile.svelte";
	import PageSection from "$lib/components/PageSection.svelte";
	import TileGrid from "$lib/components/tile_grid/TileGrid.svelte";
	import type { PageData } from "./$types";

	export let data: PageData;
</script>

<svelte:head>
	<title>Home - Media Server</title>
</svelte:head>

<main class="main-content">
	<PageSection title="Libraries">
		<TileGrid>
			{#each data.libraries as library}
				<DirectoryTile title="{library.display_name}" link="/files/{encodeURIComponent(library.id)}/" />
			{/each}
		</TileGrid>
	</PageSection>
</main>
