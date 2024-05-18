<script lang="ts">
    import { formatDuration } from "$lib/utils";
    import BaseTile from "./BaseTile.svelte";

	export let title: string;
	export let duration: number;
	export let link: string;
	export let thumbnailPath: string;
</script>

<BaseTile>
	<a class="card-link" href="{link}">
		<div class="card">
			<img class="thumbnail" src="{thumbnailPath}" alt="{title}">
			<div class="duration-container"><span>{formatDuration(duration)}</span></div>
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