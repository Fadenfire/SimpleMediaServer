type LibrariesResponse = Library[];

interface Library {
	id: string,
	display_name: string,
}

type FileInfoResponse = FileInfo | DirectoryInfo;

interface FileInfo {
	type: "file",
	display_name: String,
	size: number,
	duration: number,
	artist?: string,
}

interface DirectoryInfo {
	type: "directory",
	display_name: string,
	files: ChildFile[],
	directories: ChildDirectory[],
}

interface ChildFile {
	path_name: string,
	display_name: string,
	duration: number,
}

interface ChildDirectory {
	path_name: string,
}