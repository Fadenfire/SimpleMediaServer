<script lang="ts">
    import DirectoryTile from "$lib/components/tile_grid/DirectoryTile.svelte";
	import GridSection from "$lib/components/tile_grid/GridSection.svelte";
	import type { PageData } from "./$types";

	export let data: PageData;
</script>

<main class="main-content">
	<GridSection title="Libraries">
		{#each data.libraries as library}
			<DirectoryTile title="{library.display_name}" link="/files/{encodeURIComponent(library.id)}/" thumbnail=undefined />
		{/each}
	</GridSection>
</main>
