<script lang="ts">
    import FormattedText from "$lib/components/FormattedText.svelte";
    import { formatRichText } from "$lib/format_text";
    import dayjs from "dayjs";
    import { createEventDispatcher } from "svelte";
	
	export let comment: ApiComment;
	
	const dispatch = createEventDispatcher();
	
	$: text = formatRichText(comment.text, time => dispatch("seekTo", time));
	$: publishedAt = dayjs(comment.published_at).format("YYYY-MM-DD");
</script>

<div class="comment">
	<div class="heading">
		<span class="author">{comment.author}</span>
		<span class="date">{publishedAt}</span>
	</div>
	
	<p class="text">
		<FormattedText text={text}/>
	</p>
	
	<slot></slot>
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