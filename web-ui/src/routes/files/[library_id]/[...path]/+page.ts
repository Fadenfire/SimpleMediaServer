import { error } from "@sveltejs/kit";
import type { PageLoad } from "./$types";
import { escapePath } from "$lib/utils";

export const trailingSlash = "always";

export const load: PageLoad = async ({ params, fetch }) => {
	const res = await fetch(`/api/file_info/${encodeURIComponent(params.library_id)}/${escapePath(params.path)}`);
	
	if (res.status != 200) {
		throw error(404);
	}
	
	const file_info: FileInfoResponse = await res.json();
	
	let list_dir_promise: Promise<ListDirectoryResponse> | null = null;
	
	if (file_info.type === "directory") {
		list_dir_promise = fetch(`/api/list_dir/${encodeURIComponent(params.library_id)}/${escapePath(params.path)}`)
			.then(res => res.json());
	}
	
	return { file_info, list_dir_promise: list_dir_promise! };
}