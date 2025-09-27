<script lang="ts">
    import FeatherIcon from "./FeatherIcon.svelte";

	interface Props {
		contents?: string;
		placeholder: string;
		onCommited?: (contents: string) => void;
	}
	
	let {
		contents = $bindable(""),
		placeholder,
		onCommited,
	}: Props = $props();
</script>

<div class="box-container">
	<div class="icon"><FeatherIcon name="search" size="1em"/></div>
	<input type="text" bind:value={contents} onchange={() => onCommited?.(contents)} placeholder={placeholder}>
</div>

<style lang="scss">
	@use "input_styles.scss";
	
	.box-container {
		position: relative;
	}
	
	.icon {
		position: absolute;
		left: input_styles.$box-padding;
		top: 50%;
		transform: translateY(-40%);
	}
	
	input {
		border: none;
		
		@include input_styles.box-input;
		
		padding-left: calc(input_styles.$box-padding * 2 + 1em);
	}
</style>