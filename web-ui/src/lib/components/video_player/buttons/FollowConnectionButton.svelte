<script lang="ts">
    import { escapePath } from "$lib/utils";
    import { followConnection } from "../video_utils";
    import Button from "./Button.svelte";

	interface Props {
		connection: ApiVideoConnection;
		videoCurrentTime: number;
	}

	let { connection, videoCurrentTime }: Props = $props();
</script>

<Button large={true} tooltip="Jump to {connection.relation}" onclick={() => followConnection(connection, videoCurrentTime, -5)}>
	<img class="shortcut-thumbnail" src={escapePath(connection.shortcut_thumbnail ?? "")} alt=""/>
</Button>

<style lang="scss">
	.shortcut-thumbnail {
		object-fit: cover;
		width: 1em;
		height: 1em;
		border-radius: 50%;
		box-shadow: 0px 0px 3px 0px white;
	}
</style>