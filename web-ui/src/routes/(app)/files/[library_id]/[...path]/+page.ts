import { error } from "@sveltejs/kit";
import type { PageLoad } from "./$types";
import { escapePath } from "$lib/utils";

export const trailingSlash = "always";

export const load: PageLoad = async ({ params, fetch }) => {
	const fullUrlPath = escapePath(`${params.library_id}/${params.path}`);
	
	const res = await fetch(`/api/file_info/${fullUrlPath}`);
	
	if (res.status != 200) {
		error(404);
	}
	
	const fileInfo: FileInfoResponse = await res.json();
	
	let listDirPromise: Promise<ListDirectoryResponse> | null = null;
	
	if (fileInfo.type === "directory") {
		listDirPromise = fetch(`/api/list_dir/${fullUrlPath}`)
			.then(res => res.json());
	}
	
	return {
		fileInfo,
		listDirPromise: listDirPromise!,
	};
}