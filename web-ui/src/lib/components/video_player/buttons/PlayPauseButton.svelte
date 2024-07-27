<script lang="ts">
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import Button from "./Button.svelte";

    export let floating = false;
	
	export let videoPaused;
</script>

<Button {floating} large={floating} tooltip={videoPaused ? "Play" : "Pause"} on:click>
	{#if videoPaused}
		<FeatherIcon name="play" size="1em" style={floating ? "transform: translateX(2px);" : ""}/>
	{:else}
		<FeatherIcon name="pause" size="1em"/>
	{/if}
</Button>
