<script lang="ts">
	export let extraStyles: string | undefined = undefined;
</script>

<div class="menu" style={extraStyles}>
	<slot/>
</div>

<style lang="scss">
	@use "../player.scss";
	
	.menu {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 8px;
		overflow-y: auto;
		overflow-x: hidden;
		background-color: player.$menu-background-color;
		border-radius: 10px;
		scrollbar-color: player.$menu-foreground-color transparent;
  		scrollbar-width: thin;
	}
</style>
