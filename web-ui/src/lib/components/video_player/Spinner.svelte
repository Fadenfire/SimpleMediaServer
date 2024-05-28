<script lang="ts">
    import FeatherIcon from "../FeatherIcon.svelte";
</script>

<div class="spinner">
	<FeatherIcon name="loader"/>
</div>

<style lang="scss">
	@keyframes spin {
		from {
			transform: rotate(0turn);
		}
		
		to {
			transform: rotate(1turn);
		}
	}
	
	.spinner {
		animation: spin 2s linear 0s infinite;
		transform-origin: 50% 50%;
		width: 1em;
		height: 1em;
	}
</style>