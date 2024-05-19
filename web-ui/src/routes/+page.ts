import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ fetch }) => {
	const res = await fetch("/api/libraries");
	const libraries: Library[] = await res.json();
	
	return { libraries };
}