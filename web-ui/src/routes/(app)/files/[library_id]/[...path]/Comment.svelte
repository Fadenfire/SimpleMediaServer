<script lang="ts">
    import MultiLineText from "$lib/components/MultiLineText.svelte";
    import dayjs from "dayjs";

	export let comment: ApiComment;
	
	$: publishedAt = dayjs(comment.published_at).format("YYYY-MM-DD");
</script>

<div class="comment">
	<div class="heading">
		<span class="author">{comment.author}</span>
		<span class="date">{publishedAt}</span>
	</div>
	
	<p class="text">
		<MultiLineText text={comment.text}/>
	</p>
	
	<slot></slot>
</div>

<style lang="scss">
	.comment {
		display: flex;
		flex-direction: column;
		align-items: start;
		gap: 8px;
		// background-color: var(--foreground-inset-color);
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