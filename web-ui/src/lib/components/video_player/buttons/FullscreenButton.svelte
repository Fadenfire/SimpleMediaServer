<script lang="ts">
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import Button from "./Button.svelte";

	interface Props {
		isFullscreen: boolean;
		onclick: () => void;
	}
	
	let { isFullscreen, onclick }: Props = $props();
</script>

<Button {onclick} tooltip="Toggle Fullscreen">
	{#if isFullscreen}
		<FeatherIcon name="minimize" size="1em"/>
	{:else}
		<FeatherIcon name="maximize" size="1em"/>
	{/if}
</Button>
