<script lang="ts">
    import { page } from "$app/stores";
    import DirectoryTile from "$lib/components/tile_grid/DirectoryTile.svelte";
	import NavigationSection from "$lib/components/tile_grid/GridSection.svelte";
    import VideoTile from "$lib/components/tile_grid/VideoTile.svelte";
    import { escapePath, joinPath } from "$lib/utils";
	import type { PageData } from "./$types";

	export let data: PageData;
	
	let list_dir_promise;
	$: list_dir_promise = data.list_dir_promise!;
</script>

<svelte:head>
	<title>{data.file_info.display_name} - Media Server</title>
</svelte:head>

{#if data.file_info.type === "file"}
	<p>{data.file_info.display_name}</p>
	
	<video controls height="480" src="{`/api/media/source/${$page.params.library_id}/${escapePath($page.params.path)}`}"></video>
{:else if data.file_info.type === "directory"}
	<main class="main-content">
		{#await list_dir_promise}
			Loading
		{:then dir_list}
			<NavigationSection title="{data.file_info.display_name}">
				{#each dir_list.directories as dir}
					<DirectoryTile title="{dir.path_name}" link="{joinPath($page.url.pathname, dir.path_name)}" thumbnail="{dir.thumbnail_path ? escapePath(dir.thumbnail_path) : undefined}" child_count="{dir.child_count}" />
				{/each}
				
				{#each dir_list.files as file}
					<VideoTile
						title="{file.display_name}"
						duration="{file.duration}"
						link="{joinPath($page.url.pathname, file.path_name)}"
						thumbnailPath="{escapePath(file.thumbnail_path)}"
					/>
				{/each}
			</NavigationSection>
		{/await}
	</main>
{/if}
