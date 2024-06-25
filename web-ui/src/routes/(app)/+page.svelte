<script lang="ts">
    import DirectoryTile from "$lib/components/tile_grid/DirectoryTile.svelte";
	import PageSection from "$lib/components/PageSection.svelte";
	import TileGrid from "$lib/components/tile_grid/TileGrid.svelte";
	import type { PageData } from "./$types";
    import VideoTile from "$lib/components/tile_grid/VideoTile.svelte";
    import { escapePath } from "$lib/utils";

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
	
	<PageSection title="Watched">
		<TileGrid>
			{#each data.watch_history.entries as file}
				<VideoTile
					title="{file.display_name}"
					link="/files/{escapePath(file.full_path)}/"
					duration="{file.duration}"
					thumbnailPath="{escapePath(file.thumbnail_path)}"
					progress={file.watch_progress}
				/>
			{/each}
		</TileGrid>
	</PageSection>
</main>
