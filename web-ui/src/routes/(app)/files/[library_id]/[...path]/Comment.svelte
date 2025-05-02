<script lang="ts">
	import { type Snippet } from "svelte";

    import FormattedText from "$lib/components/FormattedText.svelte";
    import { formatRichText } from "$lib/format_text";
    import dayjs from "dayjs";
	
	interface Props {
		comment: ApiComment;
		children?: Snippet;
		seekTo: (time: number) => void;
	}

	let { comment, children, seekTo }: Props = $props();
	
	let text = $derived(formatRichText(comment.text, time => seekTo(time)));
	
	let [publishedAt, publishedAtTooltip] = $derived.by(() => {
		const date = dayjs(comment.published_at);
		
		const publishedAt = date.fromNow();
		const publishedAtTooltip = date.format("YYYY-MM-DD");
		
		return [publishedAt, publishedAtTooltip];
	});
</script>

<div class="comment">
	<div class="heading">
		<span class="author">{comment.author}</span>
		<span class="date" title={publishedAtTooltip}>{publishedAt}</span>
	</div>
	
	<p class="text">
		<FormattedText text={text}/>
	</p>
	
	{@render children?.()}
</div>

<style lang="scss">
	.comment {
		display: flex;
		flex-direction: column;
		align-items: start;
		gap: 8px;
		background-color: var(--foreground-inset-color);
		border-radius: 8px;
		padding: 8px;
		font-size: 14px;
	}
	
	.author {
		font-weight: 500;
	}
	
	.date {
		color: var(--secondary-text-color);
	}
</style>