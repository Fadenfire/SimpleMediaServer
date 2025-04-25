<script lang="ts">
	import FeatherIcon from "../FeatherIcon.svelte";
	import BaseTile from "./BaseTile.svelte";
	
	export let title: string;
	export let link: string;
	export let thumbnail: string | undefined = undefined;
	export let child_count: number = 0;
</script>

<BaseTile title="{title}" link="{link}">
	<svelte:fragment slot="card">
		<div class="thumbnail-backdrop">
			<FeatherIcon name="folder" size="4em"/>
		</div>
		
		{#if thumbnail !== undefined}
			{#key thumbnail}
				<img class="thumbnail" src="{thumbnail}" alt="{title}">
			{/key}
		{/if}
		
		{#if child_count > 0}
			<div class="count-container"><FeatherIcon name="list"/> {child_count}</div>
		{/if}
	</svelte:fragment>
</BaseTile>

<style lang="scss">
	.thumbnail-backdrop {
		position: absolute;
		left: 0;
		top: 0;
		width: 100%;
		height: 100%;
		z-index: 0;
		display: flex;
		align-items: center;
		justify-content: center;
	}
	
	.thumbnail {
		object-fit: cover;
		width: 100%;
		height: 100%;
		z-index: 1;
	}
	
	.count-container {
		position: absolute;
		left: 0px;
		bottom: 0px;
		z-index: 2;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.2em;
		background-color: rgba(#000, 0.7);
		font-size: 12px;
		padding: 3px 5px;
		width: 100%;
		text-align: center;
		font-weight: 500;
	}
</style>