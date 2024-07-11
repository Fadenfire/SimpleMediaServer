<script lang="ts">
	export let title: string;
	export let link: string | undefined = undefined;
</script>

<div class="tile">
	<a class="card-link" href={link}>
		<div class="card">
			<slot name="card"/>
		</div>
	</a>
	<div class="title-container">
		<a class="title" href={link}>{title}</a>
		<slot name="title-row"></slot>
	</div>
	<slot name="desc"></slot>
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
	
	.title-container {
		display: flex;
		flex-direction: row;
		align-items: center;
		gap: 4px;
		margin-bottom: 4px;
	}
	
	.title {
		flex: 1;
		text-decoration: none;
		color: var(--main-text-color);
		
		&[href]:hover {
			text-decoration: underline;
		}
	}
</style>
