<script lang="ts">
	interface Props {
		iconHtml: string;
		size?: string;
		width?: string;
		height?: string;
		style?: string;
		fill?: string;
	}
	
	let {
		iconHtml,
		size = "1em",
		width,
		height,
		style = "",
		fill = "none",
	}: Props = $props();
</script>

<div style="width: {width ?? size}; height: {height ?? size}; --fill: {fill}; {style}">
	{@html iconHtml}
</div>

<style lang="scss">
	div :global(svg) {
		width: 100%;
		height: 100%;
		fill: var(--fill);
	}
</style>