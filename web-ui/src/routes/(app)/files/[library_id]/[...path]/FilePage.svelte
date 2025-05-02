<script lang="ts">
	import FormattedText from "$lib/components/FormattedText.svelte";
	import VideoPlayer from "$lib/components/video_player/VideoPlayer.svelte";
    import { formatRichText } from "$lib/format_text";
    import dayjs from "dayjs";
    import Comments from "./Comments.svelte";
    import { abbreviateNumber } from "$lib/utils";
	import PathComponents from "./PathComponents.svelte";

	interface Props {
		mediaInfo: ApiFileInfo;
	}

	let { mediaInfo }: Props = $props();
	
	let videoPlayer: VideoPlayer | undefined = $state();
	
	function seekTo(time: number) {
		videoPlayer?.seekTo(time);
		window.scrollTo(0, 0);
	}
	
	let videoAspectRadio = $derived(mediaInfo.video_info ? mediaInfo.video_info.video_size.width / mediaInfo.video_info.video_size.height : 16.0 / 9.0);
	let description = $derived(mediaInfo.description ? formatRichText(mediaInfo.description, seekTo) : undefined);
		
	let extraInfo = $derived.by(() => {
		const frags = [];
		const creationDate = dayjs(mediaInfo.creation_date);
		
		if (mediaInfo.artist) frags.push(mediaInfo.artist);
		frags.push(creationDate.format("MMM D, YYYY"));
		frags.push(abbreviateNumber(mediaInfo.file_size, 1) + "B");
		
		return frags.join(" • ");
	});
</script>

<div class="main-container" style="--video-aspect-radio: {videoAspectRadio}">
	<main class="main-content">
		<section class="video-section">
			<div class="video-container">
				<VideoPlayer bind:this={videoPlayer} mediaInfo={mediaInfo}/>
			</div>
			
			<div class="info">
				<h1 class="title">{mediaInfo.display_name}</h1>
				<PathComponents info={mediaInfo}/>
				<span class="extra-info">{extraInfo}</span>
			</div>
			
			{#if description}
				<p class="description">
					<FormattedText text={description}/>
				</p>
			{/if}
		</section>
		
		{#if mediaInfo.comments.length > 0}
			<section class="comments">
				<h3>Comments</h3>
				<Comments commentThreads={mediaInfo.comments} {seekTo}/>
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
		--video-max-width: calc(100vw - 40px);
		--video-height: calc(min(var(--video-target-height), var(--video-max-width) / var(--video-aspect-radio)));
		--video-width: calc(min(var(--video-target-height) * var(--video-aspect-radio), var(--video-max-width)));
		
		width: var(--video-width);
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
	
	.info {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	
	.title {
		font-size: 24px;
	}
	
	.extra-info {
		color: var(--secondary-text-color);
		font-size: 16px;
	}
	
	.description {
		margin: 8px 0px;
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