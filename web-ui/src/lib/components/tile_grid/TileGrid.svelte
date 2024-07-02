<div class="tile-grid">
	<slot/>
</div>

<style lang="scss">
	$grid-card-width: 280px;
	$grid-tile-spacing: 30px;
	
	.tile-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax($grid-card-width, 1fr));
		gap: $grid-tile-spacing;
	}
</style>