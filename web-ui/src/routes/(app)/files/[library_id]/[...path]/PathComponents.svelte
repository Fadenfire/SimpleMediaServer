<script lang="ts">
	export let info: ApiInfoCommon;
	
	let pathComponents: string[];
	let pathComponentLinks: string[];
	
	$: {
		let url = "/files/";
		
		pathComponents = info.full_path.split("/").filter(it => it.length > 0);
		pathComponents.pop();
		
		pathComponentLinks = pathComponents.map(comp => {
			url += encodeURIComponent(comp) + "/";
			return url;
		});
		
		if (pathComponents.length > 0) pathComponents[0] = info.library_display_name;
	}
</script>

<div class="path-components">
	{#each pathComponents as pathComponent, index}
		<span class="divider">&gt;</span>
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
