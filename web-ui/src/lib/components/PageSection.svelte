<script lang="ts">
	export let title: string;
	export let titleLink: string | undefined = undefined;
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
			
			<slot name="title-bar"></slot>
		</div>
		
		<slot name="header"></slot>
	</div>
	
	<slot/>
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