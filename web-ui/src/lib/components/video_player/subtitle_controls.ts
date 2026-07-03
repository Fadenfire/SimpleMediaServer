import { AUTO_SUBTITLE_TRACK_INDEX, NO_SUBTITLE_TRACK_INDEX, type VideoPlayerState } from "./VideoPlayerInternal.svelte";
import { iso6393To1 } from "iso-639-3";
import { lookup as langLookup } from "bcp-47-match";

const SUBTITLES_ENABLED_STORAGE_KEY = "media_server_subtitles_enabled";

export function getInitialSubtitleTrack(mediaInfo: ApiFileInfo): number {
	let subtitlesEnabled = false;
	
	try {
		subtitlesEnabled = localStorage.getItem(SUBTITLES_ENABLED_STORAGE_KEY) === "true";
	} catch {}
	
	return subtitlesEnabled ? selectSubtitleTrack(mediaInfo) : NO_SUBTITLE_TRACK_INDEX;
}

export function saveSubtitlesState(subtitlesEnabled: boolean) {
	try {
		localStorage.setItem(SUBTITLES_ENABLED_STORAGE_KEY, subtitlesEnabled.toString());
	} catch {}
}

export function selectSubtitleTrack(mediaInfo: ApiFileInfo): number {
	const subtitles = mediaInfo.subtitle_streams;
	
	if (subtitles.length > 0) {
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
		
		return bestTrack.track_id;
	}
	
	if (mediaInfo.has_auto_subtitles) {
		return AUTO_SUBTITLE_TRACK_INDEX;
	}
	
	return NO_SUBTITLE_TRACK_INDEX;
}
