<script lang="ts">
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import Button from "./Button.svelte";

	interface Props {
		mediaInfo: ApiFileInfo;
		onclick: () => void;
	}

	let { mediaInfo, onclick }: Props = $props();
</script>

<Button disabled={mediaInfo.connections.length == 0} tooltip="Connections" {onclick}>
	<FeatherIcon name="link" size="1em"/>
</Button>
