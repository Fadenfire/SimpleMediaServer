<script lang="ts">
	export let label: string;
	export let value: any = undefined;
</script>

<label>
	{label}
	<div class="dropdown">
		<select bind:value>
			<slot/>
		</select>
	</div>
</label>

<style lang="scss">
	$arrows-size: 4px;
	
	label {
		display: flex;
		flex-direction: row;
		align-items: center;
		gap: 8px;
	}
	
	.dropdown {
		position: relative;
		
		&::before, &::after {
			content: "";
			position: absolute;
			right: 12px;
			top: 50%;
			pointer-events: none;
		}
		
		&::before {
			border-left: $arrows-size solid transparent;
			border-right: $arrows-size solid transparent;
			border-bottom: $arrows-size solid var(--main-text-color);
			transform: translateY(calc(-100% - 2px));
		}
		
		&::after {
			border-left: $arrows-size solid transparent;
			border-right: $arrows-size solid transparent;
			border-top: $arrows-size solid var(--main-text-color);
			transform: translateY(2px);
		}
	}
	
	select {
		appearance: none;
		-webkit-appearance: none;
		display: block;
		margin: 0;
		padding: 8px;
		padding-right: calc(3em + $arrows-size);
		background-color: var(--foreground-color);
		border: none;
		border-radius: 8px;
		color: var(--main-text-color);
		font-size: 14px;
		font-weight: inherit;
		text-decoration: none;
		cursor: pointer;
	}
</style>