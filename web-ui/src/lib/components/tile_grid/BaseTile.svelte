<script lang="ts">
	import { type Snippet } from "svelte";
	
	interface Props {
		title: string;
		link?: string;
		card?: Snippet;
		desc?: Snippet;
		descRow?: Snippet;
	}
	
	let {
		title,
		link = undefined,
		card,
		desc,
		descRow
	}: Props = $props();
</script>

<div class="tile">
	<a class="card-link" href={link}>
		<div class="card">
			{@render card?.()}
		</div>
	</a>
	<div class="card-info">
		<div class="desc-text">
			<a class="title" href={link}>{title}</a>
			{@render desc?.()}
		</div>
		{@render descRow?.()}
	</div>
</div>

<style lang="scss">
	.tile {
		display: flex;
		flex-direction: column;
	}
	
	.card-link {
		text-decoration: none;
	}
	
	.card {
		display: flex;
		align-items: center;
		justify-content: center;
		position: relative;
		aspect-ratio: 16 / 9;
		width: 100%;
		margin-bottom: 8px;
		border-radius: 8px;
		background-color: var(--foreground-color);
		color: var(--main-text-color);
		overflow: hidden;
		box-shadow: 0px 0px 8px rgba(0, 0, 0, 0.2);
	}
	
	.card-info {
		display: flex;
		flex-direction: row;
		align-items: flex-start;
		gap: 4px;
	}
	
	.desc-text {
		flex: 1;
		display: flex;
		flex-direction: column;
	}
	
	.title {
		margin-bottom: 4px;
		text-decoration: none;
		color: var(--main-text-color);
		
		&[href]:hover {
			text-decoration: underline;
		}
	}
</style>
