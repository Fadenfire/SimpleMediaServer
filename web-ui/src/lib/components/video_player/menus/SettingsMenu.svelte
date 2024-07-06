<script lang="ts">
	import type { VideoBackend } from "../video_backend";
    import SidebarMenu from "./SidebarMenu.svelte";

	export let playerBackend: VideoBackend;
	
	$: levels = playerBackend.qualityLevels;
	$: currentLevel = playerBackend.currentLevelIndex;
</script>

<SidebarMenu>
	<h3 class="title">Quality</h3>
	{#each $levels as level}
		<button class="custom-button quality-button" class:active={level.id === $currentLevel} on:click={() => $currentLevel = level.id}>
			{level.displayName}
			{#if level.desc}
				<span class="desc">({level.desc})</span>
			{/if}
		</button>
	{/each}
</SidebarMenu>

<style lang="scss">
	@use "../player.scss";
	
	.title {
		text-align: center;
	}
	
	.quality-button {
		min-width: 200px;
		text-align: center;
		background-color: player.$menu-foreground-color;
		border-radius: 10px;
		padding: 8px;
		
		&.active {
			background-color: player.$menu-bright-foreground-color;
		}
		
		.desc {
			color: var(--secondary-text-color);
		}
	}
</style>