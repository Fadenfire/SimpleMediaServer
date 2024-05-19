<script lang="ts">
	import DirectoryTile from "$lib/components/tile_grid/DirectoryTile.svelte";
	import PageSection from "$lib/components/PageSection.svelte";
    import VideoTile from "$lib/components/tile_grid/VideoTile.svelte";
    import { escapePath } from "$lib/utils";
    import TileGrid from "$lib/components/tile_grid/TileGrid.svelte";
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
	
	export let dir_info: DirectoryInfo;
	export let list_dir_promise: Promise<ListDirectoryResponse>;
</script>

<main class="main-content">
	<PageSection title="{dir_info.display_name}">
		{#await list_dir_promise}
			<div class="stripe">
				Loading
			</div>
		{:then dir_list}
			{#if !dir_list.directories.length && !dir_list.files.length}
				<div class="stripe">
					<FeatherIcon name="folder" width="3em" height="3em"/>
					Empty Directory
				</div>
			{:else}
				<TileGrid>
					{#each dir_list.directories as dir}
						<DirectoryTile
							title="{dir.path_name}"
							link="{encodeURIComponent(dir.path_name)}/"
							thumbnail="{dir.thumbnail_path ? escapePath(dir.thumbnail_path) : undefined}"
							child_count="{dir.child_count}"
						/>
					{/each}
					
					{#each dir_list.files as file}
						<VideoTile
							title="{file.display_name}"
							link="{encodeURIComponent(file.path_name)}/"
							duration="{file.duration}"
							thumbnailPath="{escapePath(file.thumbnail_path)}"
						/>
					{/each}
				</TileGrid>
			{/if}
		{/await}
	</PageSection>
</main>

<style lang="scss">
	.stripe {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5em;
		color: var(--dim-text-color);
		font-size: 22px;
	}
</style>