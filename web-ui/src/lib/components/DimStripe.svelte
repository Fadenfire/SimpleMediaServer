<div class="stripe">
	<slot/>
</div>

<style lang="scss">
	.stripe {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5em;
		color: var(--background-text-color);
		font-size: 22px;
	}
</style>