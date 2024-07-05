import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ fetch }) => {
	const librariesRes = await fetch("/api/libraries");
	const libraries: ApiLibraryEntry[] = await librariesRes.json();
	
	const watchHistoryRes = await fetch("/api/watch_history?page=0&page_size=12");
	const watchHistory: ApiWatchHistoryResponse = await watchHistoryRes.json();
	
	return { libraries, watchHistory };
}