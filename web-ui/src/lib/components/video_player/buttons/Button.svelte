<script lang="ts">
    import { createEventDispatcher } from "svelte";
	
	export let floating = false;
	export let large = false;
	export let disabled = false;
	
	const dispatch = createEventDispatcher();
	
	function onClick() {
		dispatch("click");
	}
</script>

<button class:control-button={!floating} class:floating-control-button={floating} class:large={large} class:disabled={disabled} on:click={onClick}>
	<slot/>
</button>

<style lang="scss">
	button {
		padding: 0;
		margin: 0;
		background: none;
		border-radius: 0;
		color: inherit;
		font-size: inherit;
		font-weight: inherit;
		text-decoration: none;
		border: none;
		cursor: pointer;
	}
	
	.control-button {
		width: var(--video-player-control-size);
		height: var(--video-player-control-size);
		font-size: var(--video-player-control-size);
		
		&.large {
			width: calc(var(--video-player-control-size) * 1.5);
			height: calc(var(--video-player-control-size) * 1.5);
			font-size: calc(var(--video-player-control-size) * 1.5);
		}
	}
	
	.floating-control-button {
		width: var(--video-player-center-control-size);
		height: var(--video-player-center-control-size);
		font-size: var(--video-player-center-control-icon-size);
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 50%;
		background-color: #0009;
		
		&.large {
			width: calc(var(--video-player-center-control-size) * 1.5);
			height: calc(var(--video-player-center-control-size) * 1.5);
			font-size: calc(var(--video-player-center-control-icon-size) * 1.5);
		}
	}
	
	.disabled {
		color: var(--disabled-text-color);
	}
</style>