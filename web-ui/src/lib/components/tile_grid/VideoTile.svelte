<script lang="ts">
    import { formatDuration } from "$lib/utils";
    import BaseTile from "./BaseTile.svelte";

	export let title: string;
	export let duration: number;
	export let link: string;
	export let thumbnailPath: string;
</script>

<BaseTile>
	<div class="card">
		<img class="thumbnail" src="{thumbnailPath}" alt="{title}">
		<div class="duration-container"><span>{formatDuration(duration)}</span></div>
	</div>
	<a class="title" href="{link}">{title}</a>
</BaseTile>

<style lang="scss">
	.card {
		background-color: var(--panel-color);
		width: var(--grid-card-width);
		height: calc(--grid-card-height);
		margin-bottom: 8px;
		border-radius: 8px;
		overflow: hidden;
		position: relative;
	}
	
	.thumbnail {
		object-fit: cover;
		width: 100%;
		height: 100%;
	}
	
	.duration-container {
		position: absolute;
		right: 4px;
		bottom: 4px;
		background-color: rgba(#000, 0.7);
		font-size: 12px;
		padding: 3px 5px;
		border-radius: 4px;
		font-weight: 500;
	}
	
	.title {
		text-decoration: none;
		color: var(--main-text-color);
	}
	
	.title:hover {
		text-decoration: underline;
	}
</style>