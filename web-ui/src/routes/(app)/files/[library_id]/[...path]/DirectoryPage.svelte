<script lang="ts" module>
	enum SortType {
		Name,
		DateAdded,
		Duration,
		WatchProgress,
	}
	
	enum SortDirection {
		Ascending,
		Descending,
	}
	
	export interface DirSnapshotObj {
		sortType: SortType,
		searchText: string,
	}
</script>

<script lang="ts">
	import DirectoryTile from "$lib/components/tile_grid/DirectoryTile.svelte";
	import PageSection from "$lib/components/PageSection.svelte";
    import VideoTile from "$lib/components/tile_grid/VideoTile.svelte";
    import { escapePath } from "$lib/utils";
    import TileGrid from "$lib/components/tile_grid/TileGrid.svelte";
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import DimStripe from "$lib/components/DimStripe.svelte";
    import SelectionDropdown from "$lib/components/SelectionDropdown.svelte";
	import dayjs from "dayjs";
    import PathComponents from "./PathComponents.svelte";
    import type { Snapshot } from "./$types";
    import SearchBar from "$lib/components/SearchBar.svelte";
	
	interface Props {
		dirInfo: ApiDirectoryInfo;
		listDirPromise: Promise<ListDirectoryResponse>;
	}

	let { dirInfo, listDirPromise }: Props = $props();
	
	let sortType = $state(SortType.Name);
	let searchText = $state("");
	
	export const snapshot: Snapshot<DirSnapshotObj> = {
		capture: () => {
			return {
				sortType,
				searchText,
			};
		},
		restore: (snapshot) => {
			sortType = snapshot.sortType;
			searchText = snapshot.searchText;
		}
	};
	
	let searchQuery = $derived(
		searchText
			.trim()
			.toLowerCase()
	);
	
	function sortEntries(files: ApiFileEntry[], sortType: SortType): ApiFileEntry[] {
		if (sortType == SortType.Name) return files;
		
		let keyFunc: (entry: ApiFileEntry) => any;
		let sortDirection = SortDirection.Ascending;
		
		if (sortType == SortType.DateAdded) {
			keyFunc = entry => dayjs(entry.creation_date).valueOf();
			
			sortDirection = SortDirection.Descending;
		}
		else if (sortType == SortType.Duration) {
			keyFunc = entry => entry.duration;
		}
		else if (sortType == SortType.WatchProgress) {
			keyFunc = entry => entry.watch_progress ?? 0;
		}
		
		return files.toSorted((entryA, entryB) => {
			const a = keyFunc(entryA);
			const b = keyFunc(entryB);
			
			if (a == b) {
				return 0;
			} else if (sortDirection == SortDirection.Ascending) {
				return a < b ? -1 : 1;
			} else {
				return a < b ? 1 : -1;
			}
		});
	}
	
	interface HasDisplayName {
		display_name: string;
	}
	
	function filterEntries<T extends HasDisplayName>(entries: T[], searchQuery: string): T[] {
		if (searchQuery.length == 0) return entries;
		
		return entries.filter(entry => {
			return entry.display_name.toLowerCase().includes(searchQuery);
		});
	}
</script>

<main class="main-content">
	<PageSection title={dirInfo.display_name}>
		{#snippet titleBar()}
			<SearchBar bind:contents={searchText} placeholder="Filter by name"/>
			
			<SelectionDropdown bind:value={sortType} label="Sort by">
				<option value={SortType.Name}>Name</option>
				<option value={SortType.DateAdded}>Date Added</option>
				<option value={SortType.Duration}>Duration</option>
				<option value={SortType.WatchProgress}>Watch Progress</option>
			</SelectionDropdown>
		{/snippet}
		
		{#snippet header()}
			<PathComponents info={dirInfo}/>
		{/snippet}
		
		{#await listDirPromise}
			<DimStripe>Loading</DimStripe>
			
			<!-- Hack to allow scoll position to be retained when navigating through history -->
			<div style="height: 50000px;"></div>
		{:then dirList}
			{#if !dirList.directories.length && !dirList.files.length}
				<DimStripe>
					<FeatherIcon name="folder" size="3em"/>
					Empty Directory
				</DimStripe>
			{:else}
				<TileGrid>
					{@const filteredDirs = filterEntries(dirList.directories, searchQuery)}
					
					{#each filteredDirs as dir}
						<DirectoryTile
							title={dir.path_name}
							link="{encodeURIComponent(dir.path_name)}/"
							thumbnail={dir.thumbnail_path ? escapePath(dir.thumbnail_path) : undefined}
							child_count={dir.child_count}
						/>
					{/each}
					
					{@const filteredFiles = filterEntries(dirList.files, searchQuery)}
					{@const sortedFiles = sortEntries(filteredFiles, sortType)}
					
					{#each sortedFiles as file}
						<VideoTile fileEntry={file}/>
					{/each}
				</TileGrid>
			{/if}
		{/await}
	</PageSection>
</main>
