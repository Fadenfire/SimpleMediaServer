import { goto } from "$app/navigation";
import { escapePath } from "$lib/utils";

export function jumpToVideo(skipTarget: string | null) {
	if (skipTarget === null) return;
	
	goto(`../${encodeURIComponent(skipTarget)}/`, {
		replaceState: true,
		state: {
			videoPlayerSeekTo: 0
		}
	});
}

export function calcConnectionTime(connection: ApiVideoConnection, currentTime: number): number {
	return Math.max(0, currentTime - connection.left_start + connection.right_start);
}

export function followConnection(connection: ApiVideoConnection, currentTime: number, offset: number = 0) {
	if (!(connection.left_start <= currentTime && currentTime < connection.left_end)) return;
	
	const otherTime = Math.max(0, calcConnectionTime(connection, currentTime) - offset);
	
	goto(`/files/${escapePath(connection.video_path)}/`, {
		replaceState: true,
		state: {
			videoPlayerSeekTo: otherTime
		}
	});
}
