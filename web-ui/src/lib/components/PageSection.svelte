<script lang="ts">
	import { type Snippet } from "svelte";
	
	interface Props {
		title: string;
		titleLink?: string | undefined;
		titleBar?: Snippet;
		header?: Snippet;
		children?: Snippet;
	}

	let {
		title,
		titleLink = undefined,
		titleBar,
		header,
		children
	}: Props = $props();
</script>

<section>
	<div class="header">
		<div class="title-bar">
			<h1 class="title">
				{#if titleLink}
					<a href={titleLink}>{title}</a>
				{:else}
					{title}
				{/if}
			</h1>
			
			{@render titleBar?.()}
		</div>
		
		{@render header?.()}
	</div>
	
	{@render children?.()}
</section>

<style lang="scss">
	section {
		padding: 10px;
	}
	
	.header {
		margin-bottom: 16px;
	}
	
	.title-bar {
		display: flex;
		flex-direction: row;
		align-items: center;
		gap: 8px;
	}
	
	.title {
		flex: 1;
		
		a {
			color: var(--main-text-color);
			text-decoration: none;
			
			&:hover {
				text-decoration: underline;
			}
		}
	}
</style>