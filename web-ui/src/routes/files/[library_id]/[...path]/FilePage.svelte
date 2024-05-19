<script lang="ts">
    import { page } from "$app/stores";
    import VideoPlayer from "$lib/components/video_player/VideoPlayer.svelte";

	export let file_info: FileInfo;
	
	$: video_aspect_radio = file_info.video_resolution ? file_info.video_resolution.width / file_info.video_resolution.height : 16.0 / 9.0;
</script>

<div class="main-container" style="--video-aspect-radio: {video_aspect_radio}">
	<main class="main-content">
		<div class="video-container">
			<VideoPlayer path="{`${$page.params.library_id}/${$page.params.path}`}"/>
		</div>
		<h1 class="title">{file_info.display_name}</h1>
		{#if file_info.artist} <span class="extra-info">{file_info.artist}</span> {/if}
	</main>
</div>

<style lang="scss">
	.main-container {
		display: flex;
		flex-direction: column;
		align-items: center;
	}
	
	.main-content {
		display: flex;
		flex-direction: column;
	}
	
	.video-container {
		--video-target-height: 80vh;
		--video-max-width: calc(100vw - 20px);
		height: calc(min(var(--video-target-height), var(--video-max-width) / var(--video-aspect-radio)));
		width: calc(min(var(--video-target-height) * var(--video-aspect-radio), var(--video-max-width)));
	}
	
	.title {
		font-size: 24px;
		margin: 8px 0px;
	}
	
	.extra-info {
		color: var(--secondary-text-color);
	}
</style>