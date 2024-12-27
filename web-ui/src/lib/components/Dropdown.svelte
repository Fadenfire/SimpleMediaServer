<div class="dropdown-container">
	<slot name="summary"/>
	
	<div class="dropdown">
		<slot name="dropdown"/>
	</div>
</div>

<style lang="scss">
	.dropdown-container {
		display: inline-block;
		position: relative;
	}
	
	.dropdown {
		position: absolute;
		right: 0;
		display: none;
	}
	
	.dropdown-container:hover .dropdown {
		display: block;
	}
</style>
