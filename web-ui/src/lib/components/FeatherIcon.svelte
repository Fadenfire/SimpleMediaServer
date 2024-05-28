<script lang="ts">
	import feather from "feather-icons";

	export let name: string;
	export let rotation: number = 0;
	export let size: string | undefined = undefined;
	export let width = "1em";
	export let height = "1em";
	export let strokeWidth: string | undefined = undefined;
	export let style: string = "";
	
	$: if (size) {
		width = size;
		height = size;
	}
	
	$: icon = feather.icons[name];
	$: attrs = Object.assign({}, icon.attrs);
	
	$: if (strokeWidth) {
		attrs["stroke-width"] = strokeWidth;
	}
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