<script lang="ts">
	import feather from "feather-icons";

	interface Props {
		name: string;
		rotation?: number;
		size?: string | undefined;
		width?: string;
		height?: string;
		strokeWidth?: string | undefined;
		style?: string;
	}

	let {
		name,
		rotation = 0,
		size = undefined,
		width = $bindable("1em"),
		height = $bindable("1em"),
		strokeWidth = undefined,
		style = ""
	}: Props = $props();
	
	$effect(() => {
		if (size) {
			width = size;
			height = size;
		}
	});
	
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
	style="width: {width}; height: {height}; transform: rotate({rotation}deg); {style}"
>
	<g>{@html icon.contents}</g>
</svg>

<style lang="scss">
	svg {
		overflow: visible;
		transform-origin: 50% 50%;
	}
</style>