<script lang="ts">
    import SelectionDropdown from "$lib/components/SelectionDropdown.svelte";
	import type { VideoBackend } from "../video_backend";
    import type { VideoElementState } from "../VideoElement.svelte";
    import SidebarMenu from "./SidebarMenu.svelte";

	interface Props {
		videoState: VideoElementState;
		playerBackend: VideoBackend;
		
		gamma: number;
	}

	let {
		videoState,
		playerBackend,
		
		gamma = $bindable(1.0),
	}: Props = $props();
	
	let levels = $derived(playerBackend.qualityLevels);
	let currentLevel = $derived(playerBackend.currentLevelIndex);
</script>

<SidebarMenu>
	<h3 class="title">Settings</h3>
	
	<SelectionDropdown bind:value={$currentLevel} label="Quality" style="width: 180px;">
		{#each $levels as level}
			<option value={level.id}>
				{level.displayName}
				{#if level.desc}
					({level.desc})
				{/if}
			</option>
		{/each}
	</SelectionDropdown>
	
	<SelectionDropdown bind:value={videoState.playbackRate} label="Speed" style="width: 180px;">
		<option value={0.25}>0.25x</option>
		<option value={0.5}>0.5x</option>
		<option value={0.75}>0.75x</option>
		<option value={1.0}>Normal</option>
		<option value={1.25}>1.25x</option>
		<option value={1.5}>1.5x</option>
		<option value={1.75}>1.75x</option>
		<option value={2.0}>2x</option>
	</SelectionDropdown>
	
	{#if playerBackend.mediaInfo.subtitle_streams.length > 0}
		<SelectionDropdown bind:value={videoState.subtitleTrack} label="Captions" style="width: 180px;">
			<option value={-1}>Off</option>
			
			{#each videoState.videoElement?.textTracks as track, index}
				<option value={index}>{track.label}</option>
			{/each}
		</SelectionDropdown>
	{/if}
	
	<label>
		Gamma
		<input type="range" min="1.0" max="4.0" step="0.2" bind:value={gamma}>
	</label>
</SidebarMenu>

<style lang="scss">
	@use "../player.scss";
	
	.title {
		text-align: center;
	}
</style>