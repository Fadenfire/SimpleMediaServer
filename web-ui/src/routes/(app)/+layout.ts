import type { PageLoad } from "./$types";

import { redirect } from "@sveltejs/kit";

export const load: PageLoad = async ({ fetch }) => {
	const res = await fetch("/api/get_user");
	
	if (res.status != 200 && (await res.json())?.message == "unauthorized") {
		redirect(302, "/login");
	}
	
	const userInfo: ApiUserInfo = await res.json();
	
	return { userInfo };
}