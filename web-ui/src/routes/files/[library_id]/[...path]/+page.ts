import { error } from "@sveltejs/kit";
import type { PageLoad } from "./$types";
import { escapePath } from "$lib/utils";

export const load: PageLoad = async ({ params, fetch }) => {
	const res = await fetch(`/api/file_info/${encodeURIComponent(params.library_id)}/${escapePath(params.path)}`);
	
	if (res.status != 200) {
		throw error(404);
	}
	
	const file_info: FileInfoResponse = await res.json();
	
	return { file_info };
}