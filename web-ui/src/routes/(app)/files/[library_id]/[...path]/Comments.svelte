<script lang="ts">
    import Comment from "./Comment.svelte";

	interface Props {
		commentThreads: ApiCommentThread[];
		seekTo: (time: number) => void;
	}

	let { commentThreads, seekTo }: Props = $props();
</script>

<div class="comments">
	{#each commentThreads as thread}
		<Comment comment={thread.comment} {seekTo}>
			{#if thread.replies.length > 0}
				<details>
					<summary class="replies-dropdown">{thread.replies.length} {thread.replies.length == 1 ? "Reply" : "Replies"}</summary>
					
					<div class="replies comments">
						{#each thread.replies as reply}
							<Comment comment={reply} {seekTo}/>
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
		cursor: pointer;
	}
	
	.replies {
		padding-left: 16px;
	}
</style>