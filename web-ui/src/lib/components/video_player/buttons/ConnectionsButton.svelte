<script lang="ts">
    import { goto } from "$app/navigation";
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import { escapePath } from "$lib/utils";
    import Button from "./Button.svelte";

	export let mediaInfo: ApiMediaInfo;
	export let currentTime: number;
	
	let linkTargets: ApiVideoConnection[] = [];
	
	let lastCurrentTime = -1;
	
	$: if (Math.floor(currentTime) != lastCurrentTime) {
		lastCurrentTime = Math.floor(currentTime);
		linkTargets = [];
		
		for (const connection of mediaInfo.connections) {
			if (connection.shortcut_thumbnail && connection.left_start <= currentTime && currentTime < connection.left_end) {
				linkTargets.push(connection);
			}
		}
	}
	
	function followLink(connection: ApiVideoConnection) {
		const otherTime = Math.max(0, currentTime - connection.left_start + connection.right_start - 5);
		
		goto(`/files/${escapePath(connection.other_path)}/`, {
			replaceState: true,
			state: {
				videoPlayerSeekTo: otherTime
			}
		});
	}
</script>

{#each linkTargets as connection (connection.other_path)}
	<Button large={true} on:click={() => followLink(connection)}>
		<img src="/api/thumbnail/{escapePath(connection.shortcut_thumbnail ?? "")}" alt="" class="link-thumbnail"/>
	</Button>
{/each}

<Button disabled={linkTargets.length == 0}>
	<FeatherIcon name="link" size="1em"/>
</Button>

<style lang="scss">
	.link-thumbnail {
		object-fit: cover;
		width: 100%;
		height: 100%;
		border-radius: 50%;
	}
</style>