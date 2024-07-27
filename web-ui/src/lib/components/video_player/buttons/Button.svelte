<script lang="ts">
	export let tooltip: string | undefined = undefined;
    export let floating = false;
	export let large = false;
	export let disabled = false;
	export let extraStyles = "";
</script>

<button
	class="custom-button control-button"
	class:floating={floating}
	class:large={large}
	disabled={disabled}
	style={extraStyles}
	title={tooltip}
	on:click
>
	<slot/>
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