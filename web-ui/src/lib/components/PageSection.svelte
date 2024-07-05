<script lang="ts">
	export let title: string;
	export let titleLink: string | undefined = undefined;
</script>

<section>
	<div class="title-bar">
		<h1 class="title">
			{#if titleLink}
				<a href={titleLink}>{title}</a>
			{:else}
				{title}
			{/if}
		</h1>
		
		<slot name="title-bar"></slot>
	</div>
	
	<slot/>
</section>

<style lang="scss">
	section {
		padding: 10px;
	}
	
	.title-bar {
		display: flex;
		flex-direction: row;
		align-items: center;
		margin-bottom: 10px;
	}
	
	.title {
		flex: 1;
		
		a {
			color: var(--main-text-color);
		}
	}
</style>