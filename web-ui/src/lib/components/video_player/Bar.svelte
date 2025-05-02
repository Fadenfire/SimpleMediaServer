<script lang="ts">
	import { type Snippet } from "svelte";
	
	interface Props {
		startValue?: number;
		endValue?: number;
		color: string;
		children?: Snippet;
	}

	let {
		startValue = 0,
		endValue = 1,
		color,
		children
	}: Props = $props();
</script>

<div class="bar" style="transform: translateX({startValue * 100}%) scaleX({endValue - startValue}); background-color: {color}">
	{@render children?.()}
</div>

<style lang="scss">
	.bar {
		position: absolute;
		left: 0;
		top: 0;
		width: 100%;
		height: 100%;
		transform-origin: 0 0;
	}
</style>