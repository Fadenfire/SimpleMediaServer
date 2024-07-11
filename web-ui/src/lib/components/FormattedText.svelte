<script lang="ts" context="module">
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
	export let text: FormattedText;
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
			on:click={fragment.onclick}
		>{fragment.text}</a>
	{:else}
		<span style={fragment.style}>{fragment.text}</span>
	{/if}
{/each}
