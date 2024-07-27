<script lang="ts">
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import Button from "./Button.svelte";

    export let isFullscreen;
</script>

<Button on:click tooltip="Toggle Fullscreen">
	{#if isFullscreen}
		<FeatherIcon name="minimize" size="1em"/>
	{:else}
		<FeatherIcon name="maximize" size="1em"/>
	{/if}
</Button>
