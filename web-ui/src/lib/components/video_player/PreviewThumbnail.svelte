<script lang="ts">
	interface Props {
		videoInfo: ApiVideoInfo;
		thumbSheetUrl: string;
		currentTime: number;
		extraStyles?: string;
	}

	let {
		videoInfo,
		thumbSheetUrl,
		currentTime,
		extraStyles = ""
	}: Props = $props();
	
	let [spriteX, spriteY] = $derived.by(() => {
		const offset = Math.floor(currentTime / videoInfo.thumbnail_sheet_interval);
		
		const spriteX = Math.floor(offset % videoInfo.thumbnail_sheet_cols);
		const spriteY = Math.floor(offset / videoInfo.thumbnail_sheet_rows);
		
		return [spriteX, spriteY];
	});
</script>

<div
	style="
		background-image: url({thumbSheetUrl});
		background-position: -{spriteX * 100}% -{spriteY * 100}%;
		background-size: {videoInfo.thumbnail_sheet_cols * 100}% {videoInfo.thumbnail_sheet_rows * 100}%;
		aspect-ratio: {videoInfo.sheet_thumbnail_size.width} / {videoInfo.sheet_thumbnail_size.height};
		{extraStyles}
	"
></div>
