<script lang="ts">
	import LiveTurns from './LiveTurns.svelte';
	import { stageView, statusMessage } from './stores';

	/** The running stage above the transcript: the turns on screen in the live view, and the
	 *  status line in either. The page owns the view tabs and the transcript under this. */
	let {
		statusText
	}: {
		/** The status line, as the page words it for its live region. */
		statusText: string;
	} = $props();
</script>

{#if $stageView === 'live'}
	<LiveTurns />
{/if}

<!-- The visible copy of the page's status region, which is what a screen reader hears. -->
{#if $statusMessage}
	<p class="status-msg" aria-hidden="true">{statusText}</p>
{/if}

{#if $stageView === 'live'}<span class="grow"></span>{/if}

<style>
	/* A direct child of the stage's scrolling column: it keeps its height rather than
	   compress, so a short window scrolls instead of clipping text. */
	.status-msg {
		flex: 0 0 auto;
		margin: var(--space-4) 0 0;
		font-size: var(--type-body);
		line-height: var(--leading-body);
		color: var(--warn);
	}
</style>
