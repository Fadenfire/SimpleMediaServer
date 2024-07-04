import type { PageLoad } from "./$types";

const PAGE_SIZE = 48;

export const load: PageLoad = async ({ fetch, url }) => {
	const pageIndex = parseInt(url.searchParams.get("page") ?? "0");
	
	const historyPromise: Promise<ApiWatchHistoryResponse> = fetch(`/api/watch_history?page=${pageIndex}&page_size=${PAGE_SIZE}`)
		.then(res => res.json());
	
	return { pageIndex, historyPromise };
}
