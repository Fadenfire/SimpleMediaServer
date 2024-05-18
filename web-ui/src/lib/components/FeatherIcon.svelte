<script lang="ts">
	import feather from "feather-icons";
	export const directions = ["n", "ne", "e", "se", "s", "sw", "w", "nw"] as const;

	export let name: string;
	export let direction: typeof directions[number] = "n";
	export let width = "1em";
	export let height = "1em";
	export let customClass = "";
	
	$: icon = feather.icons[name];
	$: rotation = directions.indexOf(direction) * 45;
</script>

<svg
	{...icon.attrs}
	class="{customClass}"
	style="width: {width}; height: {height}; transform: rotate({rotation}deg);"
>
	<g>{@html icon.contents}</g>
</svg>

<style lang="scss">
	svg {
		overflow: visible;
		transform-origin: 50% 50%;
	}
</style>