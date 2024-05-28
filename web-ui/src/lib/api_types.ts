type LibrariesResponse = Library[];

interface Library {
	id: string,
	display_name: string,
}

type FileInfoResponse = MediaInfo | DirectoryInfo;

interface MediaInfo {
	type: "file",
	path: string,
	display_name: String,
	file_size: number,
	duration: number,
	artist: string | null,
	video_info: VideoInfo | null,
	prev_video: string | null,
	next_video: string | null,
}

interface VideoInfo {
	video_size: Dimension,
	sheet_thumbnail_size: Dimension,
	thumbnail_sheet_rows: number,
	thumbnail_sheet_cols: number,
	thumbnail_sheet_interval: number,
}

interface DirectoryInfo {
	type: "directory",
	display_name: string,
}

interface ListDirectoryResponse {
	files: ChildFile[],
	directories: ChildDirectory[],
	total_duration: number,
}

interface ChildFile {
	path_name: string,
	display_name: string,
	thumbnail_path: string,
	duration: number,
}

interface ChildDirectory {
	path_name: string,
	display_name: string,
	thumbnail_path?: string,
	child_count: number,
}

interface Dimension {
	width: number,
	height: number,
}