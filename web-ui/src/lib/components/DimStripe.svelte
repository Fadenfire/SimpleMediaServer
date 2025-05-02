<script lang="ts">
	import { type Snippet } from "svelte";
	
	interface Props {
		children?: Snippet;
	}

	let { children }: Props = $props();
</script>

<div class="stripe">
	{@render children?.()}
</div>

<style lang="scss">
	.stripe {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5em;
		color: var(--background-text-color);
		font-size: 22px;
	}
</style>