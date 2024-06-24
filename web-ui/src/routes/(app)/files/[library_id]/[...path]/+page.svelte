<script lang="ts">
    import type { PageData } from "./$types";
    import DirectoryPage from "./DirectoryPage.svelte";
    import FilePage from "./FilePage.svelte";

	export let data: PageData;
</script>

<svelte:head>
	<title>{data.fileInfo.display_name} - Media Server</title>
</svelte:head>

{#if data.fileInfo.type === "file"}
	<FilePage mediaInfo={data.fileInfo}/>
{:else if data.fileInfo.type === "directory"}
	<DirectoryPage dirInfo={data.fileInfo} listDirPromise={data.listDirPromise}/>
{/if}
