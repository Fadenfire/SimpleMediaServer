<script lang="ts">
    import Button from "./Button.svelte";
    import SVGIcon from "$lib/components/SVGIcon.svelte";
	
	import PiPInnerIcon from "$lib/icons/inner-pip.svg?raw";
	import PiPInnerIconFilled from "$lib/icons/inner-pip-filled.svg?raw";
    
    interface Props {
		hasConnections: boolean;
		toggled: boolean;
		onclick: () => void;
	}
	
	let {
		hasConnections,
		toggled,
		onclick
	}: Props = $props();
	
</script>

<Button disabled={!hasConnections} tooltip="Toggle Connections PiP" {onclick}>
	<SVGIcon iconHtml={toggled ? PiPInnerIconFilled : PiPInnerIcon} size="1em"/>
</Button>
