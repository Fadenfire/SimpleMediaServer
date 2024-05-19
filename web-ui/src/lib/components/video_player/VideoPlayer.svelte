<script lang="ts">
    import { escapePath } from "$lib/utils";

	export let path: string;
</script>

<figure class="player-container">
	<!-- svelte-ignore a11y-media-has-caption -->
	<video controls src="{`/api/media/source/${escapePath(path)}`}"></video>
	
</figure>

<style lang="scss">
	.player-container {
		width: 100%;
		height: 100%;
		background-color: black;
	}
	
	video {
		width: 100%;
		height: 100%;
	}
</style>