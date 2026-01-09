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
		
		const textTracks = videoState.videoElement.textTracks;
		
		if (videoState.subtitlesEnabled()) {
			videoState.subtitleTrack = -1;
		} else if (textTracks.length > 0) {
			const langCodeToTrack = new Map();
			
			for (let i = 0; i < textTracks.length; i++) {
				const track = textTracks[i];
				const lang = iso6393To1[track.language] ?? track.language;
				
				// If there are multiple tracks with the same language
				//  use the first one
				if (!langCodeToTrack.has(lang)) {
					langCodeToTrack.set(lang, i);
				}
			}
			
			const bestMatch = langLookup(langCodeToTrack.keys().toArray(), Array.from(navigator.languages));
			
			let bestTrack = 0;
			
			if (bestMatch !== undefined) {
				bestTrack = langCodeToTrack.get(bestMatch) ?? bestTrack;
			}
			
			videoState.subtitleTrack = bestTrack;
		}
	}
</script>

<Button tooltip="Closed Captions" onclick={onClick}>
	<SVGIcon iconHtml={videoState.subtitlesEnabled() ? CCIconSolid : CCIconRegular} size="1em"/>
</Button>
