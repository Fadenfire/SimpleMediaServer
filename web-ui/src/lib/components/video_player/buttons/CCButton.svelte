<script lang="ts">
    import Button from "./Button.svelte";
    import SVGIcon from "$lib/components/SVGIcon.svelte";
    import type { VideoElementState } from "../VideoElement.svelte";
	import { iso6393To1 } from "iso-639-3";
	import { lookup as langLookup } from "bcp-47-match";
	
	import CCIconRegular from "$lib/icons/closed-captioning-regular-full.svg?raw";
	import CCIconSolid from "$lib/icons/closed-captioning-solid-full.svg?raw";
    
    interface Props {
		videoState: VideoElementState;
	}
	
	let { videoState }: Props = $props();
	
	function onClick() {
		if (videoState.videoElement === undefined) return;
		
		const subtitles = videoState.mediaInfo.subtitle_streams;
		
		if (videoState.subtitlesEnabled()) {
			videoState.subtitleTrack = -1;
		} else if (subtitles.length > 0) {
			const langCodeToTrack = new Map<string, ApiSubtitleStream>();
			
			for (const track of subtitles) {
				if (track.language === null) continue;
				
				const lang = iso6393To1[track.language] ?? track.language;
				
				// If there are multiple tracks with the same language
				//  use the first one
				if (!langCodeToTrack.has(lang)) {
					langCodeToTrack.set(lang, track);
				}
			}
			
			const bestMatch = langLookup(langCodeToTrack.keys().toArray(), Array.from(navigator.languages));
			
			let bestTrack = subtitles[0];
			
			if (bestMatch !== undefined) {
				bestTrack = langCodeToTrack.get(bestMatch) ?? bestTrack;
			}
			
			videoState.subtitleTrack = bestTrack.track_id;
		}
	}
</script>

<Button tooltip="Closed Captions" onclick={onClick}>
	<SVGIcon iconHtml={videoState.subtitlesEnabled() ? CCIconSolid : CCIconRegular} size="1em"/>
</Button>
