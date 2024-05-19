<script lang="ts">
    import type { PageData } from "./$types";
    import DirectoryPage from "./DirectoryPage.svelte";
    import FilePage from "./FilePage.svelte";

	export let data: PageData;
</script>

<svelte:head>
	<title>{data.file_info.display_name} - Media Server</title>
</svelte:head>

{#if data.file_info.type === "file"}
	<FilePage file_info={data.file_info}/>
{:else if data.file_info.type === "directory"}
	<DirectoryPage dir_info={data.file_info} list_dir_promise={data.list_dir_promise}/>
{/if}
