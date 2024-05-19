<div class="tile-grid">
	<slot/>
</div>

<style lang="scss">
	.tile-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(var(--grid-card-width), 1fr));
		gap: var(--grid-tile-spacing);
	}
</style>