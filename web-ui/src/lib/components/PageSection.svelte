<script lang="ts">
	export let title: string;
	export let titleLink: string | undefined = undefined;
</script>

<section>
	<h1>
		{#if titleLink}
			<a href={titleLink}>{title}</a>
		{:else}
			{title}
		{/if}
	</h1>
	<slot/>
</section>

<style lang="scss">
	section {
		padding: 10px;
	}
	
	h1 {
		margin-bottom: 10px;
		
		a {
			color: var(--main-text-color);
		}
	}
</style>