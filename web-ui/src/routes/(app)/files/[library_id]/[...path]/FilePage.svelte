<script lang="ts">
    import MultiLineText from "$lib/components/MultiLineText.svelte";
	import VideoPlayer from "$lib/components/video_player/VideoPlayer.svelte";
    import Comments from "./Comments.svelte";

	export let mediaInfo: ApiFileInfo;
	
	$: videoAspectRadio = mediaInfo.video_info ? mediaInfo.video_info.video_size.width / mediaInfo.video_info.video_size.height : 16.0 / 9.0;
</script>

<div class="main-container" style="--video-aspect-radio: {videoAspectRadio}">
	<main class="main-content">
		<section class="video-section">
			<div class="video-container">
				<VideoPlayer mediaInfo={mediaInfo}/>
			</div>
			
			<h1 class="title">{mediaInfo.display_name}</h1>
			
			{#if mediaInfo.artist}
				<span class="extra-info">{mediaInfo.artist}</span>
			{/if}
			
			{#if mediaInfo.description}
				<p class="description">
					<MultiLineText text={mediaInfo.description}/>
				</p>
			{/if}
		</section>
		
		{#if mediaInfo.comments.length > 0}
			<section class="comments">
				<h3>Comments</h3>
				<Comments commentThreads={mediaInfo.comments}/>
			</section>
		{/if}
	</main>
</div>

<style lang="scss">
	.main-container {
		display: flex;
		flex-direction: column;
		align-items: center;
	}
	
	.main-content {
		--video-target-height: 80vh;
		--video-max-width: calc(100vw - 20px);
		--video-height: calc(min(var(--video-target-height), var(--video-max-width) / var(--video-aspect-radio)));
		--video-width: calc(min(var(--video-target-height) * var(--video-aspect-radio), var(--video-max-width)));
		
		width: var(--video-width);
		overflow-x: hidden;
	}
	
	.video-section {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	
	.video-container {
		width: var(--video-width);
		height: var(--video-height);
	}
	
	.basic-info {
		display: flex;
		flex-direction: column;
	}
	
	.title {
		font-size: 24px;
	}
	
	.extra-info {
		color: var(--secondary-text-color);
	}
	
	.description {
		padding: 12px;
		border-radius: 8px;
		background-color: var(--foreground-inset-color);
		font-size: 14px;
	}
	
	.comments {
		margin: 24px 0px;
	}
	
	h3 {
		line-height: 2em;
	}
</style>