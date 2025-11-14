<script lang="ts" module>
	const GAMMA_FILTER_PARAMS = [
		[2.000000, 0.452152, 0.462721, 0.498724], // gamma = 1.0
        [2.000000, 0.499219, 0.495900, 0.467721], // gamma = 1.1
        [2.000000, 0.507297, 0.491958, 0.476888], // gamma = 1.2
        [2.000000, 0.366565, 0.347899, 0.633254], // gamma = 1.3
        [2.000000, 0.508557, 0.473272, 0.505025], // gamma = 1.4
        [2.000000, 0.527114, 0.482141, 0.499587], // gamma = 1.5
        [2.000000, 0.542055, 0.486633, 0.498278], // gamma = 1.6
        [2.000000, 0.558441, 0.490201, 0.497107], // gamma = 1.7
        [2.000000, 0.575215, 0.493272, 0.496148], // gamma = 1.8
        [2.000000, 0.592294, 0.495900, 0.495383], // gamma = 1.9
        [2.000000, 0.609619, 0.498143, 0.494787], // gamma = 2.0
        [1.988238, 0.630085, 0.496656, 0.497379], // gamma = 2.1
        [1.924253, 0.663724, 0.479544, 0.514163], // gamma = 2.2
        [1.873559, 0.695524, 0.465286, 0.528181], // gamma = 2.3
        [1.828081, 0.726615, 0.451691, 0.541521], // gamma = 2.4
        [1.789154, 0.756722, 0.439525, 0.553466], // gamma = 2.5
        [1.755775, 0.785932, 0.428671, 0.564131], // gamma = 2.6
        [1.724339, 0.814763, 0.417952, 0.574635], // gamma = 2.7
        [1.696790, 0.842908, 0.408240, 0.584156], // gamma = 2.8
        [1.672707, 0.870461, 0.399494, 0.592735], // gamma = 2.9
        [1.651378, 0.897450, 0.391548, 0.600539], // gamma = 3.0
        [1.632313, 0.924123, 0.384284, 0.607668], // gamma = 3.1
        [1.615464, 0.950346, 0.377735, 0.614109], // gamma = 3.2
        [1.598456, 0.976416, 0.370882, 0.620815], // gamma = 3.3
        [1.586391, 1.000000, 0.366060, 0.625773], // gamma = 3.4
        [1.619375, 1.000000, 0.381577, 0.613567], // gamma = 3.5
        [1.539013, 0.868777, 0.284548, 0.707899], // gamma = 3.6
        [1.527966, 0.675506, 0.213132, 0.781071], // gamma = 3.7
        [1.517953, 0.813632, 0.247715, 0.745398], // gamma = 3.8
        [1.507017, 0.899316, 0.263884, 0.728565], // gamma = 3.9
        [1.498849, 0.988431, 0.280737, 0.711087], // gamma = 4.0
        [1.494799, 0.974058, 0.270376, 0.722089], // gamma = 4.1
        [1.486346, 0.904763, 0.243132, 0.749944], // gamma = 4.2
        [1.478595, 0.899141, 0.234215, 0.758984], // gamma = 4.3
        [1.471300, 0.826058, 0.208768, 0.785055], // gamma = 4.4
        [1.464979, 0.646923, 0.158903, 0.836322], // gamma = 4.5
        [1.459022, 0.980698, 0.234308, 0.758548], // gamma = 4.6
        [1.453584, 0.463082, 0.107731, 0.888941], // gamma = 4.7
        [1.448625, 0.978679, 0.221914, 0.771149], // gamma = 4.8
        [1.442303, 0.483678, 0.106678, 0.889921], // gamma = 4.9
        [1.438229, 1.000000, 0.215379, 0.777694], // gamma = 5.0
	];
</script>

<script lang="ts">
    import type { Snippet } from "svelte";
	import { randomId } from "$lib/utils";

	interface Props {
		children: Snippet;
		
		gamma: number;
	}
	
	let {
		children,
		gamma,
	}: Props = $props();
	
	const isSafari = /^((?!chrome|android).)*safari/i.test(navigator.userAgent);
	
	let params = $derived.by(() => {
		if (gamma <= 1) return "";
		
		const lookup_index = Math.round(gamma * 10) - 10;
		const [u, h, g, k] = GAMMA_FILTER_PARAMS[lookup_index];
		
		return `--const-u: ${u}; --const-h: ${h}; --const-g: ${g}; --const-k: ${k};`;
	});
		
	const svgFilterPrefix = randomId();
	const avgGammaFilterId = `${svgFilterPrefix}-gammaFilter`;
</script>

{#if isSafari}
	<div class="filters-container" class:gamma-filter={gamma > 1} style={params}>
		<div class="backdrop-l1 absolute"></div>
		<div class="content-l1">
			<div class="backdrop-l2 absolute"></div>
			<div class="content-l2">
				{@render children()}
			</div>
		</div>
	</div>
{:else}
	<div class="filters-container" style={gamma > 1 ? `filter: url(#${avgGammaFilterId});` : undefined}>
		{@render children()}
		
		<svg width="0" height="0" xmlns="http://www.w3.org/2000/svg">
			<filter id="{avgGammaFilterId}">
				<feComponentTransfer color-interpolation-filters="sRGB">
					<feFuncR type="linear" slope="-1.0" intercept="1.0" />
					<feFuncG type="linear" slope="-1.0" intercept="1.0" />
					<feFuncB type="linear" slope="-1.0" intercept="1.0" />
				</feComponentTransfer>
				
				<feComponentTransfer color-interpolation-filters="sRGB">
					<feFuncR type="gamma" amplitude="1.0" exponent="{gamma}" offset="0.0" />
					<feFuncG type="gamma" amplitude="1.0" exponent="{gamma}" offset="0.0" />
					<feFuncB type="gamma" amplitude="1.0" exponent="{gamma}" offset="0.0" />
				</feComponentTransfer>
				
				<feComponentTransfer color-interpolation-filters="sRGB">
					<feFuncR type="linear" slope="-1.0" intercept="1.0" />
					<feFuncG type="linear" slope="-1.0" intercept="1.0" />
					<feFuncB type="linear" slope="-1.0" intercept="1.0" />
				</feComponentTransfer>
			</filter>
		</svg>
	</div>
{/if}

<style lang="scss">
	div {
		width: 100%;
		height: 100%;
	}
	
	.filters-container {
		position: relative;
		isolation: isolate;
	}
	
	.absolute {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
	}
	
	.gamma-filter {
		&.filters-container {
			filter: brightness(var(--const-u));
		}
		
		.backdrop-l1 {
			background-color: color(display-p3 var(--const-k) var(--const-k) var(--const-k));
		}
		
		.content-l1 {
			mix-blend-mode: color-burn;
		}
		
		.backdrop-l2 {
			background-color: color(display-p3 1 1 1);
		}
		
		.content-l2 {
			filter: brightness(var(--const-h));
			opacity: calc(1 - var(--const-g));
		}
	}
</style>