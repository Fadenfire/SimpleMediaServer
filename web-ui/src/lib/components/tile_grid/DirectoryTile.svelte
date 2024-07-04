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
		{#if thumbnail !== undefined}
			<img class="thumbnail" src="{thumbnail}" alt="{title}">
		{:else}
			<FeatherIcon name="folder" size="4em"/>
		{/if}
		
		{#if child_count > 0}
			<div class="count-container"><FeatherIcon name="list"/> {child_count}</div>
		{/if}
	</svelte:fragment>
</BaseTile>

<style lang="scss">
	.thumbnail {
		object-fit: cover;
		width: 100%;
		height: 100%;
	}
	
	.count-container {
		position: absolute;
		left: 0px;
		bottom: 0px;
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