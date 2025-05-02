<script lang="ts">
	import { type Snippet } from "svelte";
	
	interface Props {
		extraStyles?: string | undefined;
		children?: Snippet;
	}

	let { extraStyles = undefined, children }: Props = $props();
</script>

<div class="menu" style={extraStyles}>
	{@render children?.()}
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
