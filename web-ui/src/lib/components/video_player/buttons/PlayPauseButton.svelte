<script lang="ts">
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import Button from "./Button.svelte";
	
	interface Props {
		floating?: boolean;
		videoPaused: any;
		onclick: () => void;
	}
	
	let { floating = false, videoPaused, onclick }: Props = $props();
</script>

<Button {floating} large={floating} tooltip={videoPaused ? "Play" : "Pause"} {onclick}>
	{#if videoPaused}
		<FeatherIcon name="play" size="1em" style={floating ? "transform: translateX(2px);" : ""}/>
	{:else}
		<FeatherIcon name="pause" size="1em"/>
	{/if}
</Button>
