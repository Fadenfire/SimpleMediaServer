<script lang="ts">
	export let title: string;
</script>

<section>
	<h1>{title}</h1>
	<div class="tile-grid">
		<slot></slot>
	</div>
</section>

<style lang="scss">
	section {
		padding: 10px;
	}
	
	h1 {
		margin-bottom: 10px;
	}
	
	.tile-grid {
		display: flex;
		flex-wrap: wrap;
		align-items: start;
		gap: var(--grid-tile-spacing);
	}
</style>