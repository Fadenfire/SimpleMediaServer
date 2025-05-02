<script lang="ts">
    import { goto } from "$app/navigation";
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import Button from "./Button.svelte";

	interface Props {
		floating?: boolean;
		direction: "forward" | "back";
		mediaInfo: ApiFileInfo;
	}

	let { floating = false, direction, mediaInfo }: Props = $props();
	
	let skipTarget = $derived(direction === "forward" ? mediaInfo.next_video : mediaInfo.prev_video);
	
	function onClick() {
		if (skipTarget !== null) {
			goto(`../${encodeURIComponent(skipTarget)}/`, {
				replaceState: true,
				state: {
					videoPlayerSeekTo: 0
				}
			});
		}
	}
</script>

<Button {floating} disabled={skipTarget === null} tooltip={direction === "forward" ? "Next Video" : "Previous Video"} onclick={onClick}>
	{#if direction === "forward"}
		<FeatherIcon name="skip-forward" size="1em"/>
	{:else}
		<FeatherIcon name="skip-back" size="1em"/>
	{/if}
</Button>
