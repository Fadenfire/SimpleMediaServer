interface ApiUserInfo {
	display_name: string,
	username: string,
}

type LibrariesResponse = ApiLibrary[];

interface ApiLibrary {
	id: string,
	display_name: string,
}

type FileInfoResponse = ApiMediaInfo | ApiDirectoryInfo;

interface ApiMediaInfo {
	type: "file",
	path: string,
	display_name: String,
	file_size: number,
	duration: number,
	artist: string | null,
	video_info: ApiVideoInfo | null,
	prev_video: string | null,
	next_video: string | null,
	watch_progress: number | null,
}

interface ApiVideoInfo {
	video_size: ApiDimension,
	sheet_thumbnail_size: ApiDimension,
	thumbnail_sheet_rows: number,
	thumbnail_sheet_cols: number,
	thumbnail_sheet_interval: number,
}

interface ApiDirectoryInfo {
	type: "directory",
	display_name: string,
}

interface ListDirectoryResponse {
	files: ApiChildFile[],
	directories: ApiChildDirectory[],
	total_duration: number,
}

interface ApiChildFile {
	path_name: string,
	display_name: string,
	thumbnail_path: string,
	duration: number,
	watch_progress: number | null,
}

interface ApiChildDirectory {
	path_name: string,
	display_name: string,
	thumbnail_path: string | null,
	child_count: number,
}

interface ApiDimension {
	width: number,
	height: number,
}

interface UpdateWatchProgressParams {
	library_id: string,
	media_path: string,
	new_watch_progress: number,
}
