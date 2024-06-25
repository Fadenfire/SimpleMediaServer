export function formatDuration(time: number): string {
	let hours = Math.floor(time / 3600);
	let minutes = Math.floor(time % 3600 / 60).toString();
	let seconds = Math.floor(time % 60).toString().padStart(2, "0");
	
	if (hours > 0) {
		minutes = minutes.padStart(2, "0");
		
		return `${hours}:${minutes}:${seconds}`;
	} else {
		return `${minutes}:${seconds}`;
	}
}

export function joinPath(path: string, component: string): string {
	if (!path.endsWith("/")) {
		path += "/";
	}
	
	return path + encodeURIComponent(component);
}

export function escapePath(path: string): string {
	return path.split("/").map(encodeURIComponent).join("/");
}

export function splitLibraryPath(libraryPath: string): [string, string] {
	const slashPos = libraryPath.indexOf("/");
	
	return slashPos == -1 ? [libraryPath, ""] : [libraryPath.slice(0, slashPos), libraryPath.slice(slashPos + 1)];
}

export function isStandalone(): boolean {
	return window.matchMedia("(display-mode: standalone)").matches;
}

export function replayAnimations(elem: Element | undefined) {
	if (elem === undefined) return;
	
	elem.getAnimations().forEach(anim => {
		anim.cancel();
		anim.play();
	});
}