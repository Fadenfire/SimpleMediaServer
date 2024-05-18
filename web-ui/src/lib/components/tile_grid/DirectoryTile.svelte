<script lang="ts">
	import FeatherIcon from "../FeatherIcon.svelte";
	import BaseTile from "./BaseTile.svelte";
	
	export let title: string;
	export let link: string;
	export let thumbnail: string | undefined;
	export let child_count: number = 0;
</script>

<BaseTile>
	<a class="card-link" href="{link}">
		<div class="card">
			{#if thumbnail !== undefined}
				<img class="thumbnail" src="{thumbnail}" alt="{title}">
			{:else}
				<div class="folder-icon"><FeatherIcon name="folder" width="4em" height="4em"/></div>
			{/if}
			{#if child_count > 0}
				<div class="count-container"><FeatherIcon name="list"/> {child_count}</div>
			{/if}
		</div>
	</a>
	<a class="title" href="{link}">{title}</a>
</BaseTile>

<style lang="scss">
	.card {
		background-color: var(--panel-color);
		width: var(--grid-card-width);
		height: var(--grid-card-height);
		margin-bottom: 8px;
		border-radius: 8px;
		color: var(--main-text-color);
		overflow: hidden;
		position: relative;
		box-shadow: 0px 0px 8px rgba(0, 0, 0, 0.2);
	}
	
	.folder-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 100%;
		height: 100%;
	}
	
	.thumbnail {
		object-fit: cover;
		width: 100%;
		height: 100%;
	}
	
	.count-container {
		position: absolute;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.2em;
		left: 0px;
		bottom: 0px;
		background-color: rgba(#000, 0.7);
		font-size: 12px;
		padding: 3px 5px;
		width: 100%;
		text-align: center;
		font-weight: 500;
	}
	
	.card-link {
		text-decoration: none;
	}
	
	.title {
		text-decoration: none;
		color: var(--main-text-color);
	}
	
	.title:hover {
		text-decoration: underline;
	}
</style>