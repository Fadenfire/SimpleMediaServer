<script lang="ts" module>
	export const NEWLINE: unique symbol = Symbol("newline");
	
	export interface FormattedTextFragment {
		text: string,
		style?: string,
		href?: string,
		onclick?: () => boolean | void,
	}
	
	export type FormattedText = (string | typeof NEWLINE | FormattedTextFragment)[];
</script>

<script lang="ts">
	interface Props {
		text: FormattedText;
	}

	let { text }: Props = $props();
</script>

{#each text as fragment}
	{#if typeof(fragment) === "string"}
		{fragment}
	{:else if fragment === NEWLINE}
		<br/>
	{:else if fragment.href !== undefined}
		<a
			href={fragment.href}
			class="normal-link"
			style={fragment.style}
			onclick={fragment.onclick}
		>{fragment.text}</a>
	{:else}
		<span style={fragment.style}>{fragment.text}</span>
	{/if}
{/each}
