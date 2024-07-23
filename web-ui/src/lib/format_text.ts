import { NEWLINE, type FormattedText } from "./components/FormattedText.svelte";
import { parseDuration } from "./utils";

const TIMESTAMP_PATTERN = /\b(?:\d+:)?\d{1,2}:\d{1,2}\b/;

const TOKEN_REGEX = new RegExp(`(${TIMESTAMP_PATTERN.source}|\\n|\\s+)`, "g");
const TIMESTAMP_REGEX = new RegExp(`^${TIMESTAMP_PATTERN.source}$`);

export function formatRichText(fullText: string, seekTo?: (time: number) => void): FormattedText {
	const fragments: FormattedText = fullText.split(TOKEN_REGEX)
		.map(frag => {
			if (frag == "\n") {
				return NEWLINE;
			} else if (seekTo && TIMESTAMP_REGEX.test(frag)) {
				const time = parseDuration(frag);
				
				return {
					text: frag,
					href: "javascript:;",
					onclick: () => seekTo(time),
				};
			}
			
			try {
				const url = new URL(frag);
				
				if (url.hostname.length > 0) {
					return {
						text: frag,
						href: url.href,
					};
				}
			} catch {}
			
			return frag;
		});
	
	return collapseStrings(fragments);
}

function collapseStrings(text: FormattedText): FormattedText {
	const result: FormattedText = [];
	let curText = "";
	
	for (const frag of text) {
		if (typeof(frag) === "string") {
			curText += frag;
		} else {
			if (curText.length > 0) result.push(curText);
			curText = "";
			
			result.push(frag);
		}
	}
	
	if (curText.length > 0) result.push(curText);
	
	return result;
}
