import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ url }) => {
	const pageIndex = parseInt(url.searchParams.get("page") ?? "0");
	
	return { pageIndex };
}
