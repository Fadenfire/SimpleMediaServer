<script>
    import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    import { isStandalone } from "$lib/utils";

</script>
<nav>
	<a class="nav-item" href="/"><FeatherIcon name="home" width="100%" height="100%"/></a>
	
	{#if isStandalone()}
		<a class="nav-item" href="javascript:window.history.back()"><FeatherIcon name="arrow-left" width="100%" height="100%"/></a>
	{/if}
</nav>

<slot/>

<style lang="scss">
	nav {
		display: flex;
		width: 100%;
		background-color: var(--foreground-color);
		color: var(--main-text-color);
	}
	
	.nav-item {
		width: var(--nav-bar-height);
		height: var(--nav-bar-height);
		color: var(--main-text-color);
		text-decoration: none;
		padding: 8px;
	}
	
	.nav-item:hover {
		background-color: var(--foreground-inset-color);
	}
</style>