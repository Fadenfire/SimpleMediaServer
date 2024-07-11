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

export function parseDuration(text: string): number {
	const e = text.split(":");
	e.reverse();
	
	return parseInt(e[0]) + parseInt(e[1]) * 60 + parseInt(e?.[2] ?? "0") * 60 * 60;
}

const POWER_UNITS = ["k", "M", "G", "T", "E"];

export function abbreviateNumber(num: number, digits: number = 0) {
	const power = Math.floor(Math.log10(num) / 3);
	if (power <= 0) return num.toFixed();
	
	const x = num / Math.pow(1000, power);
	const unit = POWER_UNITS?.[power - 1] ?? "?";
	
	return x.toFixed(digits) + unit;
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
