<script lang="ts">
	interface Props {
		iconHtml: string;
		size?: string | undefined;
		width?: string;
		height?: string;
	}
	
	let {
		iconHtml,
		size = undefined,
		width = $bindable("1em"),
		height = $bindable("1em"),
	}: Props = $props();
	
	$effect(() => {
		if (size) {
			width = size;
			height = size;
		}
	});
</script>

<div style="width: {width}; height: {height};">
	{@html iconHtml}
</div>

<style lang="scss">
	div :global(svg) {
		width: 100%;
		height: 100%;
		fill: currentColor;
	}
</style>