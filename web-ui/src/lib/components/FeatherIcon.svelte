<script lang="ts">
	import feather from "feather-icons";

	interface Props {
		name: string;
		rotation?: number;
		size?: string;
		width?: string;
		height?: string;
		strokeWidth?: string;
		style?: string;
	}

	let {
		name,
		rotation = 0,
		size = "1em",
		width,
		height,
		strokeWidth,
		style = "",
	}: Props = $props();
	
	let icon = $derived(feather.icons[name]);
	let attrs = $derived(Object.assign({}, icon.attrs));
	
	$effect(() => {
		if (strokeWidth) {
			attrs["stroke-width"] = strokeWidth;
		}
	});
</script>

<svg
	{...attrs}
	style="width: {width ?? size}; height: {height ?? size}; transform: rotate({rotation}deg); {style}"
>
	<g>{@html icon.contents}</g>
</svg>

<style lang="scss">
	svg {
		overflow: visible;
		transform-origin: 50% 50%;
	}
</style>