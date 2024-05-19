<script lang="ts">
	export let title: string;
</script>

<section>
	<h1>{title}</h1>
	<slot/>
</section>

<style lang="scss">
	section {
		padding: 10px;
	}
	
	h1 {
		margin-bottom: 10px;
	}
</style>