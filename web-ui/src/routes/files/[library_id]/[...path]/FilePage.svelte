<script lang="ts">
    import { page } from "$app/stores";
    import { escapePath } from "$lib/utils";

	export let file_info: FileInfo;
</script>

<p>{file_info.display_name}</p>

<!-- svelte-ignore a11y-media-has-caption -->
<video controls height="480" src="{`/api/media/source/${$page.params.library_id}/${escapePath($page.params.path)}`}"></video>