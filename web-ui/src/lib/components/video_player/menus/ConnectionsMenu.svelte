<script lang="ts">
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import { escapePath, formatDuration } from "$lib/utils";
    import Button from "../buttons/Button.svelte";
    import { followLink } from "../buttons/ConnectionsButton.svelte";
    import SidebarMenu from "./SidebarMenu.svelte";

	export let mediaInfo: ApiMediaInfo;
	export let videoElement: HTMLVideoElement;
	export let videoCurrentTime: number;
	
	function seekTo(time: number) {
		videoCurrentTime = time;
		videoElement.currentTime = time;
	}
</script>

<SidebarMenu>
	{#each mediaInfo.connections as connection}
		{@const active = connection.left_start <= videoCurrentTime && videoCurrentTime < connection.left_end}
		
		<div class="link-entry" class:active>
			<button class="custom-button" on:click={() => seekTo(connection.left_start)}>
				<img class="link-thumbnail" src="{escapePath(connection.video_thumbnail)}" alt=""/>
			</button>
			
			<div class="desc">
				<h5 class="title">{connection.relation}</h5>
				<span class="info">{formatDuration(connection.left_start)} - {formatDuration(connection.left_end)}</span>
				<span class="info">{formatDuration(connection.right_start)} - {formatDuration(connection.right_start + (connection.left_end - connection.left_start))}</span>
			</div>
			
			<Button disabled={!active} on:click={() => followLink(connection, videoCurrentTime)}>
				<FeatherIcon name="external-link" size="1em"/>
			</Button>
		</div>
	{/each}
</SidebarMenu>

<style lang="scss">
	@use "../player.scss";
	
	.link-entry {
		display: flex;
		gap: 8px;
		width: 280px;
		background-color: player.$menu-foreground-color;
		border-radius: 10px;
		padding: 8px;
		
		&.active {
			background-color: player.$menu-bright-foreground-color;
		}
		
		.desc {
			flex: 1;
			display: flex;
			flex-direction: column;
		}
		
		.title {
			font-weight: bold;
		}
		
		.info {
			font-size: 12px;
		}
	}
	
	.link-thumbnail {
		height: 50px;
	}
</style>