<script lang="ts">
	export let title: string;
	export let link: string;
</script>

<div class="tile">
	<a class="card-link" href="{link}">
		<div class="card">
			<slot name="card" />
		</div>
	</a>
	<a class="title" href="{link}">{title}</a>
	<slot name="desc"></slot>
</div>

<style lang="scss">
	.card-link {
		text-decoration: none;
	}
	
	.card {
		background-color: var(--foreground-color);
		aspect-ratio: 16 / 9;
		width: 100%;
		margin-bottom: 8px;
		border-radius: 8px;
		color: var(--main-text-color);
		overflow: hidden;
		position: relative;
		box-shadow: 0px 0px 8px rgba(0, 0, 0, 0.2);
	}
	
	.title {
		text-decoration: none;
		color: var(--main-text-color);
	}
	
	.title:hover {
		text-decoration: underline;
	}
</style>
