<script lang="ts" context="module">
	export function followLink(connection: ApiVideoConnection, currentTime: number) {
		if (!(connection.left_start <= currentTime && currentTime < connection.left_end)) return;
		
		const otherTime = Math.max(0, currentTime - connection.left_start + connection.right_start - 5);
		
		goto(`/files/${escapePath(connection.video_path)}/`, {
			replaceState: true,
			state: {
				videoPlayerSeekTo: otherTime
			}
		});
	}
</script>

<script lang="ts">
    import { goto } from "$app/navigation";
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import { escapePath } from "$lib/utils";
    import Button from "./Button.svelte";

	export let mediaInfo: ApiFileInfo;
	export let videoCurrentTime: number;
	
	$: shortcutLinks = mediaInfo.connections.filter(connection => connection.shortcut_thumbnail !== null);
	
	let linkTargets: ApiVideoConnection[] = [];
	let lastCurrentTime = -1;
	
	$: if (Math.floor(videoCurrentTime) != lastCurrentTime) {
		lastCurrentTime = Math.floor(videoCurrentTime);
		linkTargets = shortcutLinks.filter(connection => connection.left_start <= videoCurrentTime && videoCurrentTime < connection.left_end);
	}
</script>

{#each linkTargets as connection (connection.video_path)}
	<Button large={true} on:click={() => followLink(connection, videoCurrentTime)}>
		<img class="shortcut-thumbnail" src="{escapePath(connection.shortcut_thumbnail ?? "")}" alt=""/>
	</Button>
{/each}

<Button disabled={mediaInfo.connections.length == 0} on:click>
	<FeatherIcon name="link" size="1em"/>
</Button>

<style lang="scss">
	.shortcut-thumbnail {
		object-fit: cover;
		width: 1em;
		height: 1em;
		border-radius: 50%;
	}
</style>