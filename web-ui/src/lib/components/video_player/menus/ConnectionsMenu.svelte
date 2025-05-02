<script lang="ts">
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import { escapePath, formatDuration } from "$lib/utils";
    import Button from "../buttons/Button.svelte";
    import { followLink } from "../buttons/ConnectionsButton.svelte";
    import SidebarMenu from "./SidebarMenu.svelte";

	interface Props {
		mediaInfo: ApiFileInfo;
		videoElement: HTMLVideoElement | undefined;
		videoCurrentTime: number;
	}

	let { mediaInfo, videoElement = $bindable(), videoCurrentTime = $bindable() }: Props = $props();
	
	function seekTo(time: number) {
		if (videoElement === undefined) return;
		
		videoCurrentTime = time;
		videoElement.currentTime = time;
	}
	
	// $: connectedVideos = Array.from(new Set(mediaInfo.connections.map(con => con.video_path)))
</script>

<SidebarMenu>
	<!-- <div style="display: flex; gap: 10px; height: 100%;">
		{#each connectedVideos as connectedVideo}
			{@const joe = mediaInfo.connections.filter(con => con.video_path == connectedVideo)}
			<div>
				{joe[0].relation}
				<div style="height: 100%; width: 4px; background-color: blue; position: relative;">
					{#each joe as connection}
						<div style="background-color: red; position: absolute; left: 0; top: 0; width: 100%; height: 100%; transform-origin: 0 0; transform: translateY({connection.left_start / mediaInfo.duration * 100}%) scaleY({(connection.left_end - connection.left_start) / mediaInfo.duration});"></div>
					{/each}
				</div>
			</div>
		{/each}
	</div> -->
	
	{#each mediaInfo.connections as connection}
		{@const active = connection.left_start <= videoCurrentTime && videoCurrentTime < connection.left_end}
		
		<div class="link-entry" class:active>
			<button class="custom-button" onclick={() => seekTo(connection.left_start)}>
				<img class="link-thumbnail" src="{escapePath(connection.video_thumbnail)}" alt=""/>
			</button>
			
			<div class="desc">
				<h5 class="title">{connection.relation}</h5>
				<span class="info">{formatDuration(connection.left_start)} - {formatDuration(connection.left_end)}</span>
				<span class="info">{formatDuration(connection.right_start)} - {formatDuration(connection.right_start + (connection.left_end - connection.left_start))}</span>
			</div>
			
			<Button disabled={!active} onclick={() => followLink(connection, videoCurrentTime)}>
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