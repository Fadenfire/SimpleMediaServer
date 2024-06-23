import { error } from "@sveltejs/kit";
import type { PageLoad } from "./$types";
import { escapePath } from "$lib/utils";

export const trailingSlash = "always";

export const load: PageLoad = async ({ params, fetch }) => {
	const res = await fetch(`/api/file_info/${encodeURIComponent(params.library_id)}/${escapePath(params.path)}`);
	
	if (res.status != 200) {
		throw error(404);
	}
	
	const fileInfo: FileInfoResponse = await res.json();
	
	let listDirPromise: Promise<ListDirectoryResponse> | null = null;
	
	if (fileInfo.type === "directory") {
		listDirPromise = fetch(`/api/list_dir/${encodeURIComponent(params.library_id)}/${escapePath(params.path)}`)
			.then(res => res.json());
	}
	
	return { fileInfo, listDirPromise: listDirPromise! };
}