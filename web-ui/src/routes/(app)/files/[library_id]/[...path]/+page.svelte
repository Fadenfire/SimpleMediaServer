<script lang="ts" context="module">
	interface SnapshotObj {
		dirSnap: DirSnapshotObj | undefined,
	}
</script>

<script lang="ts">
    import type { PageData, Snapshot } from "./$types";
    import DirectoryPage, { type DirSnapshotObj } from "./DirectoryPage.svelte";
    import FilePage from "./FilePage.svelte";

	export let data: PageData;
	
	let dirPage: DirectoryPage | undefined;
	
	export const snapshot: Snapshot<SnapshotObj> = {
		capture: () => ({
			dirSnap: dirPage?.snapshot?.capture(),
		}),
		restore: (snapshot) => {
			if (dirPage && snapshot.dirSnap) dirPage.snapshot.restore(snapshot.dirSnap);
		}
	};
</script>

<svelte:head>
	<title>{data.fileInfo.display_name} - Media Server</title>
</svelte:head>

{#if data.fileInfo.type === "file"}
	<FilePage mediaInfo={data.fileInfo}/>
{:else if data.fileInfo.type === "directory"}
	<DirectoryPage bind:this={dirPage} dirInfo={data.fileInfo} listDirPromise={data.listDirPromise}/>
{/if}
