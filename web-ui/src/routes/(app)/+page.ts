import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ fetch }) => {
	const libraries_res = await fetch("/api/libraries");
	const libraries: ApiLibrary[] = await libraries_res.json();
	
	const watch_history_res = await fetch("/api/watch_history?page=0&page_size=12");
	const watch_history: ApiWatchHistoryResponse = await watch_history_res.json();
	
	return { libraries, watch_history };
}