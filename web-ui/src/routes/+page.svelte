<script lang="ts">
    import type { PageData } from "./$types";

	export let data: PageData;
</script>

<ul>
	{#each data.libraries as library}
		<li>{library.display_name} - {library.id}</li>
	{/each}
</ul>
