<script lang="ts">
    import Button from "./Button.svelte";
    import SVGIcon from "$lib/components/SVGIcon.svelte";
    import { NO_SUBTITLE_TRACK_INDEX, type VideoPlayerState } from "../VideoPlayerInternal.svelte";

	import CCIconRegular from "$lib/icons/closed-captioning-regular-full.svg?raw";
	import CCIconSolid from "$lib/icons/closed-captioning-solid-full.svg?raw";
    import { selectSubtitleTrack } from "../subtitle_controls";

    interface Props {
		playerState: VideoPlayerState;
	}

	let { playerState }: Props = $props();

	function onClick() {
		if (playerState.videoState.videoElement === undefined) return;

		if (playerState.subtitlesEnabled()) {
			playerState.subtitleTrack = NO_SUBTITLE_TRACK_INDEX;
		} else {
			playerState.subtitleTrack = selectSubtitleTrack(playerState.mediaInfo);
		}
	}
</script>

<div class="cc-button">
	<Button tooltip="Closed Captions" onclick={onClick}>
		<SVGIcon iconHtml={playerState.subtitlesEnabled() ? CCIconSolid : CCIconRegular} size="1em" fill="currentColor"/>
	</Button>

	{#if playerState.autoSubtitleLoading}
		<span class="loading-dot" title="Loading subtitles"></span>
	{/if}
</div>

<style lang="scss">
	.cc-button {
		position: relative;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.loading-dot {
		position: absolute;
		transform-origin: center;
		bottom: -0.05em;
		left: 0.5em;

		width: 0.2em;
		height: 0.2em;
		border-radius: 50%;
		background-color: currentColor;

		animation: bounce 0.5s ease-in-out infinite alternate;
	}

	@keyframes bounce {
		from {
			transform: translateX(0.4em);
		}
		to {
			transform: translateX(-0.4em);
		}
	}
</style>
