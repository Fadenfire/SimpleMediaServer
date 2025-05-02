<script lang="ts">
	interface Props {
		info: ApiInfoCommon;
	}

	let { info }: Props = $props();
	
	let [pathComponents, pathComponentLinks] = $derived.by(() => {
		let url = "/files/";
		
		const pathComponents = info.full_path.split("/").filter(it => it.length > 0);
		pathComponents.pop();
		
		const pathComponentLinks = pathComponents.map(comp => {
			url += encodeURIComponent(comp) + "/";
			return url;
		});
		
		if (pathComponents.length > 0) pathComponents[0] = info.library_display_name;
		
		return [pathComponents, pathComponentLinks];
	});
</script>

<div class="path-components">
	{#each pathComponents as pathComponent, index}
		{#if index != 0} <span class="divider">&gt;</span> {/if}
		<a class="normal-link" href={pathComponentLinks[index]}>{pathComponent}</a>
	{/each}
</div>

<style lang="scss">
	.path-components {
		display: flex;
		gap: 6px;
		font-size: 14px;
	}
	
	.divider {
		font-weight: bold;
	}
</style>
