<script lang="ts">
    import { page } from "$app/stores";
    import DirectoryTile from "$lib/components/tile_grid/DirectoryTile.svelte";
	import NavigationSection from "$lib/components/tile_grid/GridSection.svelte";
    import VideoTile from "$lib/components/tile_grid/VideoTile.svelte";
    import { escapePath, joinPath } from "$lib/utils";
	import type { PageData } from "./$types";

	export let data: PageData;
</script>

{#if data.file_info.type === "file"}
	<p>{data.file_info.display_name}</p>
	
	<video controls height="480" src="{`/api/media/source/${$page.params.library_id}/${escapePath($page.params.path)}`}"></video>
{:else if data.file_info.type === "directory"}
	<main class="main-content">
		<NavigationSection title="{data.file_info.display_name}">
			{#each data.file_info.directories as dir}
				<DirectoryTile title="{dir.path_name}" link="{joinPath($page.url.pathname, dir.path_name)}" />
			{/each}
			
			{#each data.file_info.files as file}
				<VideoTile
					title="{file.display_name}"
					duration="{file.duration}"
					link="{joinPath($page.url.pathname, file.path_name)}"
					thumbnailPath="{`/api/thumbnail/${$page.params.library_id}/${joinPath($page.params.path, file.path_name)}`}"
				/>
			{/each}
		</NavigationSection>
	</main>
{/if}
