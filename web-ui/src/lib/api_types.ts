// Responses

type LibrariesResponse = ApiLibraryEntry[];

type FileInfoResponse = (ApiFileInfo & { type: "file" }) | (ApiDirectoryInfo & { type: "directory" });

interface ListDirectoryResponse {
	files: ApiFileEntry[],
	directories: ApiDirectoryEntry[],
	total_duration: number,
	total_size: number,
}

interface ApiWatchHistoryResponse {
	total_pages: number,
	entries: ApiWatchHistoryEntry[],
}

// Params

interface UpdateWatchProgressParams {
	library_id: string,
	media_path: string,
	new_watch_progress: number,
}

interface DeleteWatchProgressParams {
	library_id: string,
	media_path: string,
}

// Shared Types

interface ApiUserInfo {
	display_name: string,
	username: string,
}

interface ApiLibraryEntry {
	id: string,
	display_name: string,
}

interface ApiFileEntry {
	path_name: string,
	full_path: string,
	display_name: string,
	thumbnail_path: string,
	duration: number,
	file_size: number,
	artist: string | null,
	watch_progress: number | null,
	creation_date: string,
}

interface ApiDirectoryEntry {
	path_name: string,
	display_name: string,
	thumbnail_path: string | null,
	child_count: number,
}

interface ApiInfoCommon {
	full_path: string,
	library_display_name: string,
	display_name: string,
}

interface ApiFileInfo extends ApiInfoCommon {
	file_size: number,
	duration: number,
	artist: string | null,
	creation_date: string,
	video_info: ApiVideoInfo | null,
	prev_video: string | null,
	next_video: string | null,
	watch_progress: number | null,
	description: string | null,
	connections: ApiVideoConnection[],
	comments: ApiCommentThread[],
}

interface ApiVideoInfo {
	video_size: ApiDimension,
	sheet_thumbnail_size: ApiDimension,
	thumbnail_sheet_rows: number,
	thumbnail_sheet_cols: number,
	thumbnail_sheet_interval: number,
}

interface ApiVideoConnection {
	video_path: string,
	video_thumbnail: string,
	relation: string,
	shortcut_thumbnail: string | null,
	left_start: number,
	left_end: number,
	right_start: number,
}

interface ApiCommentThread {
	comment: ApiComment,
	replies: ApiComment[],
}

interface ApiComment {
	author: string,
	text: string,
	likes: number,
	published_at: string,
}

interface ApiDirectoryInfo extends ApiInfoCommon {}

interface ApiWatchHistoryEntry {
	library_id: string,
	media_path: string,
	last_watched: string,
	progress: number,
	file: ApiFileEntry | null,
}

interface ApiDimension {
	width: number,
	height: number,
}
