<script lang="ts">
	export let videoInfo: ApiVideoInfo;
	export let thumbSheetUrl: string;
	export let currentTime: number;
	
	export let extraStyles = "";
	
	let spriteX = 0;
	let spriteY = 0;
	
	$: {
		const offset = Math.floor(currentTime / videoInfo.thumbnail_sheet_interval);
		
		spriteX = Math.floor(offset % videoInfo.thumbnail_sheet_cols);
		spriteY = Math.floor(offset / videoInfo.thumbnail_sheet_rows);
	}
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
