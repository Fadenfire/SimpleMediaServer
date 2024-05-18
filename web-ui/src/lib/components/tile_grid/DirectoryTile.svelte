<script lang="ts">
	import BaseTile from "./BaseTile.svelte";
	
	export let title: string;
	export let link: string;
	export let thumbnail: string | undefined;
</script>

<BaseTile>
	<a href="{link}"><div class="card"></div></a>
	<a class="title" href="{link}">{title}</a>
</BaseTile>

<style lang="scss">
	.card {
		background-color: var(--panel-color);
		width: var(--grid-card-width);
		height: calc(--grid-card-height);
		margin-bottom: 8px;
		border-radius: 8px;
	}
	
	.title {
		text-decoration: none;
		color: var(--main-text-color);
	}
	
	.title:hover {
		text-decoration: underline;
	}
</style>