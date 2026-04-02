<script lang="ts">
    import { onMount } from "svelte";
    import VideoElement, { VideoState } from "./VideoElement.svelte";
    import Spinner from "./Spinner.svelte";
    import { calcConnectionTime, followConnection } from "./video_utils";
    import { escapePath } from "$lib/utils";
    import { LevelType } from "./video_backend";

	interface Props {
		connection: ApiVideoConnection;
		parentVideoState: VideoState;
	}
	
	let {
		connection,
		parentVideoState,
	}: Props = $props();
	
	let videoState: VideoState = $state(null!);
	
	let videoAspectRadio = $derived(
		videoState.videoElement !== undefined && videoState.videoElement.videoWidth > 0 && videoState.videoElement.videoHeight > 0 ?
		videoState.videoElement.videoWidth / videoState.videoElement.videoHeight :
		16.0 / 9.0
	);
	
	let parentPaused = $derived(parentVideoState.isPaused || parentVideoState.isBuffering || parentVideoState.isEnded);
	
	$effect(() => {
		videoState.isPaused = parentPaused;
	});
	
	$effect(() => {
		videoState.playbackRate = parentVideoState.playbackRate;
	});
	
	$effect(() => {
		const targetTime = calcConnectionTime(connection, parentVideoState.currentTime);
		
		if (videoState.videoElement && Math.abs(videoState.currentTime - targetTime) > 0.5) {
			videoState.currentTime = targetTime;
			videoState.videoElement.currentTime = targetTime;
			
			// Just to make sure, also sync paused state
			videoState.isPaused = parentPaused;
			videoState.playbackRate = parentVideoState.playbackRate;
		}
	});
	
	onMount(() => {
		if (videoState.videoElement === undefined) throw Error("Video element is undefined");
		
		videoState.videoElement.muted = true;
		
		const startTime = calcConnectionTime(connection, parentVideoState.currentTime);
		
		videoState.currentTime = startTime;
		videoState.videoElement.currentTime = startTime;
		
		videoState.isPaused = parentPaused;
		videoState.playbackRate = parentVideoState.playbackRate;
		
		// For some reason I have to explicitly call pause() to keep it from
		//  playing
		if (parentPaused) videoState.videoElement.pause();
		
		videoState.playerBackend?.qualityLevels.subscribe(levels => {
			if (levels.length > 0) {
				const selectedLevel = levels.find(level =>
					level.levelType === LevelType.HlsManual &&
					level.hlsVideoHeight !== undefined &&
					level.hlsVideoHeight < 300
				);
				
				if (selectedLevel !== undefined) {
					console.log(`Setting PiP window to quality level ${selectedLevel.displayName}`)
					videoState.playerBackend?.currentLevelIndex.set(selectedLevel.id);
				}
			}
		});
	});
	
	function onClick(event: Event) {
		event.stopPropagation();
		
		followConnection(connection, parentVideoState.currentTime);
	}
</script>

<div
	class="pip-card"
	onclick={onClick}
	onkeydown={(e) => { if (e.key === "Enter") onClick(e); }}
	role="button"
	tabindex="0"
>
	<div class="pip-header">
		<span class="pip-label">{connection.relation}</span>
		<img class="pip-icon" src={escapePath(connection.shortcut_thumbnail ?? "")} alt=""/>
	</div>
	
	<div class="pip-video-container" style="aspect-ratio: {videoAspectRadio};">
		<VideoElement
			mediaPath={connection.video_path}
			provideState={state => videoState = state}
		/>
		
		{#if videoState.isBuffering}
			<div class="pip-spinner-container">
				<Spinner/>
			</div>
		{/if}
	</div>
</div>

<style lang="scss">
	@use "player.scss";
	
	.pip-card {
		max-width: 200px;
		background: rgba(0, 0, 0, 0.7);
		border-radius: 8px;
		overflow: hidden;
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.6);
		border: 1px solid rgba(255, 255, 255, 0.12);
		cursor: pointer;
	}
	
	.pip-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 2px 4px;
	}
	
	.pip-label {
		flex: 1;
		font-size: 10px;
		font-weight: 600;
		color: rgba(255, 255, 255, 0.9);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	
	.pip-icon {
		object-fit: cover;
		width: 1em;
		height: 1em;
		border-radius: 50%;
	}
	
	.pip-video-container {
		display: flex;
		position: relative;
	}

	.pip-spinner-container {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		width: 30px;
		height: 30px;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 20px;
		border-radius: 50%;
		background-color: player.$floating-background-color;
	}
</style>