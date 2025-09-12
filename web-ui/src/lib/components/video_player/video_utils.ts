import { goto } from "$app/navigation";

export function jumpToVideo(skipTarget: string | null) {
	if (skipTarget === null) return;
	
	goto(`../${encodeURIComponent(skipTarget)}/`, {
		replaceState: true,
		state: {
			videoPlayerSeekTo: 0
		}
	});
}
