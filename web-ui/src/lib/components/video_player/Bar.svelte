<script lang="ts">
	export let startValue = 0;
	export let endValue = 1;
	export let color: string;
</script>

<div class="bar" style="transform: translateX({startValue * 100}%) scaleX({endValue - startValue}); background-color: {color}">
	<slot></slot>
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