<script lang="ts">
    import Comment from "./Comment.svelte";

	export let commentThreads: ApiCommentThread[];
</script>

<div class="comments">
	{#each commentThreads as thread}
		<Comment comment={thread.comment} on:seekTo>
			{#if thread.replies.length > 0}
				<details>
					<summary class="replies-dropdown">{thread.replies.length} {thread.replies.length == 1 ? "Reply" : "Replies"}</summary>
					
					<div class="replies comments">
						{#each thread.replies as reply}
							<Comment comment={reply}/>
						{/each}
					</div>
				</details>
			{/if}
			
		</Comment>
	{/each}
</div>

<style lang="scss">
	.comments {
		display: flex;
		flex-direction: column;
		align-items: start;
		gap: 12px;
	}
	
	.replies-dropdown {
		font-weight: 500;
		color: var(--link-color);
	}
	
	.replies {
		padding-left: 16px;
	}
</style>