<script lang="ts">
	import { type Snippet } from "svelte";
	
	interface Props {
		summary?: Snippet;
		dropdown?: Snippet;
	}

	let { summary, dropdown }: Props = $props();
</script>

<div class="dropdown-container">
	{@render summary?.()}
	
	<div class="dropdown">
		{@render dropdown?.()}
	</div>
</div>

<style lang="scss">
	.dropdown-container {
		display: inline-block;
		position: relative;
	}
	
	.dropdown {
		position: absolute;
		right: 0;
		display: none;
	}
	
	.dropdown-container:hover .dropdown {
		display: block;
	}
</style>
