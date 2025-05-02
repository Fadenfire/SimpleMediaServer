<script lang="ts">
	import { type Snippet } from "svelte";

	interface Props {
		tooltip?: string | undefined;
		floating?: boolean;
		large?: boolean;
		disabled?: boolean;
		extraStyles?: string;
		children?: Snippet;
		onclick: () => void;
	}

	let {
		tooltip = undefined,
		floating = false,
		large = false,
		disabled = false,
		extraStyles = "",
		children,
		onclick,
	}: Props = $props();
</script>

<button
	class="custom-button control-button"
	class:floating={floating}
	class:large={large}
	disabled={disabled}
	style={extraStyles}
	title={tooltip}
	onclick={() => onclick()}
>
	{@render children?.()}
</button>

<style lang="scss">
	@use "../player.scss";
	
	button {
		display: flex;
		align-items: center;
		justify-content: center;
	}
	
	button[disabled] {
		color: var(--disabled-text-color);
		cursor: auto;
	}
	
	:not(.floating) {
		width: var(--video-player-control-size);
		height: var(--video-player-control-size);
		font-size: var(--video-player-control-size);
		
		&.large {
			width: var(--video-player-large-control-size);
			height: var(--video-player-large-control-size);
			font-size: var(--video-player-large-control-size);
		}
	}
	
	.floating {
		@include player.floating-circle;
		
		&.large {
			@include player.floating-circle-large;
		}
	}
</style>