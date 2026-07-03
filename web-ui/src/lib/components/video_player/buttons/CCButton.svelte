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

<Button tooltip="Closed Captions" onclick={onClick}>
	<SVGIcon iconHtml={playerState.subtitlesEnabled() ? CCIconSolid : CCIconRegular} size="1em" fill="currentColor"/>
</Button>
