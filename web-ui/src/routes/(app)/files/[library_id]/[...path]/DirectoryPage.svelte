<script lang="ts">
	import DirectoryTile from "$lib/components/tile_grid/DirectoryTile.svelte";
	import PageSection from "$lib/components/PageSection.svelte";
    import VideoTile from "$lib/components/tile_grid/VideoTile.svelte";
    import { escapePath } from "$lib/utils";
    import TileGrid from "$lib/components/tile_grid/TileGrid.svelte";
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
	
	export let dirInfo: ApiDirectoryInfo;
	export let listDirPromise: Promise<ListDirectoryResponse>;
</script>

<main class="main-content">
	<PageSection title="{dirInfo.display_name}">
		{#await listDirPromise}
			<div class="stripe">
				Loading
			</div>
		{:then dirList}
			{#if !dirList.directories.length && !dirList.files.length}
				<div class="stripe">
					<FeatherIcon name="folder" width="3em" height="3em"/>
					Empty Directory
				</div>
			{:else}
				<TileGrid>
					{#each dirList.directories as dir}
						<DirectoryTile
							title="{dir.path_name}"
							link="{encodeURIComponent(dir.path_name)}/"
							thumbnail="{dir.thumbnail_path ? escapePath(dir.thumbnail_path) : undefined}"
							child_count="{dir.child_count}"
						/>
					{/each}
					
					{#each dirList.files as file}
						<VideoTile
							title="{file.display_name}"
							link="{encodeURIComponent(file.path_name)}/"
							duration="{file.duration}"
							thumbnailPath="{escapePath(file.thumbnail_path)}"
							progress={file.watch_progress}
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