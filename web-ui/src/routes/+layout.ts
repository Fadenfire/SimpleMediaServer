import type { PageLoad } from "./$types";

export const ssr = false;

import "$lib/css/global.scss"

export const load: PageLoad = async ({ fetch }) => {
	const res = await fetch("/api/get_user");
	const userInfo: ApiUserInfo = await res.json();
	
	return { userInfo };
}