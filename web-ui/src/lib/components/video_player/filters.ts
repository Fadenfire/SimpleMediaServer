import { createDataURI } from "$lib/utils";

export function createLighteningFilter(exponent: number): string {
	const svg = `
<svg width="0" height="0" xmlns="http://www.w3.org/2000/svg">
	<filter id="videoFilter">
		<feComponentTransfer color-interpolation-filters="sRGB">
			<feFuncR type="linear" slope="-1.0" intercept="1.0" />
			<feFuncG type="linear" slope="-1.0" intercept="1.0" />
			<feFuncB type="linear" slope="-1.0" intercept="1.0" />
		</feComponentTransfer>
		
		<feComponentTransfer color-interpolation-filters="sRGB">
			<feFuncR type="gamma" amplitude="1.0" exponent="${exponent}" offset="0.0" />
			<feFuncG type="gamma" amplitude="1.0" exponent="${exponent}" offset="0.0" />
			<feFuncB type="gamma" amplitude="1.0" exponent="${exponent}" offset="0.0" />
		</feComponentTransfer>
		
		<feComponentTransfer color-interpolation-filters="sRGB">
			<feFuncR type="linear" slope="-1.0" intercept="1.0" />
			<feFuncG type="linear" slope="-1.0" intercept="1.0" />
			<feFuncB type="linear" slope="-1.0" intercept="1.0" />
		</feComponentTransfer>
	</filter>
</svg>
`;
	
	const dataURI = createDataURI("image/svg+xml", svg);
	
	return `url("${dataURI}#videoFilter")`;
}