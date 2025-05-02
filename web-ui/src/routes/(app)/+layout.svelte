<script lang="ts">
	import { type Snippet } from "svelte";
	
    import Dropdown from "$lib/components/Dropdown.svelte";
	import FeatherIcon from "$lib/components/FeatherIcon.svelte";
    // import { isStandalone } from "$lib/utils";
    import type { PageData } from "./$types";
	
	interface Props {
		data: PageData;
		children?: Snippet;
	}

	let { data, children }: Props = $props();
</script>

<nav>
	<a class="nav-item" href="/"><FeatherIcon name="home" size="100%"/></a>
	<a class="nav-item" href="/watch_history"><FeatherIcon name="clock" size="100%"/></a>
	
	<!-- svelte-ignore a11y_invalid_attribute -->
	<a class="nav-item" href="javascript:window.history.back()"><FeatherIcon name="arrow-left" size="100%"/></a>
	
	<div style="flex: 1;"></div>
	
	<Dropdown>
		{#snippet summary()}
			<div class="nav-item" >{data.userInfo.display_name}</div>
		{/snippet}
		
		{#snippet dropdown()}
			<div  class="dropdown-menu">
				<a class="nav-item" href="/api/logout">Logout</a>
			</div>
		{/snippet}
	</Dropdown>
</nav>

{@render children?.()}

<style lang="scss">
	$nav-bar-height: 40px;
	
	nav {
		display: flex;
		width: 100%;
		background-color: var(--foreground-color);
		color: var(--main-text-color);
	}
	
	.nav-item {
		display: flex;
		align-items: center;
		justify-content: center;
		height: $nav-bar-height;
		color: var(--main-text-color);
		text-decoration: none;
		padding: 8px;
	}
	
	.nav-item:hover {
		background-color: var(--foreground-inset-color);
	}
	
	.dropdown-menu {
		display: flex;
		flex-direction: column;
		background-color: var(--foreground-color);
		min-width: 100px;
	}
</style>