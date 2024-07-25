<script lang="ts">
    import { goto } from "$app/navigation";
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import Button from "./Button.svelte";

    export let floating = false;
	export let direction: "forward" | "back";
	
	export let mediaInfo: ApiFileInfo;
	
	$: skipTarget = direction === "forward" ? mediaInfo.next_video : mediaInfo.prev_video;
	
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

<Button {floating} disabled={skipTarget === null} on:click={onClick}>
	{#if direction === "forward"}
		<FeatherIcon name="skip-forward" size="1em"/>
	{:else}
		<FeatherIcon name="skip-back" size="1em"/>
	{/if}
</Button>
