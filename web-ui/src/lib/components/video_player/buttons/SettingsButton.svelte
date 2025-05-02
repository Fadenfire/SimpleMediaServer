<script lang="ts">
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import Button from "./Button.svelte";
    
    interface Props {
		onclick: () => void;
	}
	
	let { onclick }: Props = $props();
</script>

<Button tooltip="Settings" {onclick}>
	<FeatherIcon name="settings" size="0.9em"/>
</Button>
