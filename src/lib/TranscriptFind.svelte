<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from './i18n';
	import { ariaKeyShortcut } from './shortcuts';
	import ToolButton from './ui/ToolButton.svelte';

	/** The find bar over the transcript reader: a field, how many matches and which one is
	 *  current, and the way through them. Enter and Shift+Enter step, Escape closes and hands
	 *  focus back to where it came from. The searching itself is the log's. */
	let {
		query = $bindable(''),
		total,
		current,
		focusRequest,
		onStep,
		onClose
	}: {
		query?: string;
		/** How many matches there are for the query as last searched; null before one. */
		total: number | null;
		/** The current match, from 0; -1 for none. */
		current: number;
		/** Changes each time the window asks for the field again (Ctrl+F while open). */
		focusRequest: number;
		onStep: (step: 1 | -1) => void;
		/** Close, handing focus back to `returnTo` when it is still there. */
		onClose: (returnTo: HTMLElement | null) => void;
	} = $props();

	let field = $state<HTMLInputElement | null>(null);
	// Where focus was when find opened, to give it back on Escape.
	let returnTo: HTMLElement | null = null;
	const id = $props.id();

	onMount(() => {
		const active = document.activeElement;
		returnTo = active instanceof HTMLElement && active !== document.body ? active : null;
	});

	$effect(() => {
		void focusRequest;
		field?.focus();
		field?.select();
	});

	const countText = $derived(
		total === null || !query.trim() ? '' : $t.transcript.find.count(current + 1, total)
	);

	function onKeydown(event: KeyboardEvent) {
		if (event.isComposing) return;
		if (event.key === 'Enter') {
			event.preventDefault();
			onStep(event.shiftKey ? -1 : 1);
		} else if (event.key === 'Escape') {
			event.preventDefault();
			event.stopPropagation();
			onClose(returnTo);
		} else if ((event.ctrlKey || event.metaKey) && event.code === 'KeyF') {
			// Already here: select what is typed, and keep the browser's own find bar shut.
			event.preventDefault();
			field?.select();
		}
	}
</script>

<div class="find" role="search">
	<input
		bind:this={field}
		bind:value={query}
		type="search"
		class="field"
		aria-label={$t.transcript.find.field}
		aria-describedby="{id}-count"
		aria-keyshortcuts={ariaKeyShortcut('find')}
		autocomplete="off"
		spellcheck="false"
		onkeydown={onKeydown}
	/>
	<!-- The count, for the eye; the same words go to the status region below, which is in the
	     DOM from the moment the bar opens, so the first count is heard. -->
	<span class="count" id="{id}-count" aria-hidden="true">{countText}</span>
	<ToolButton
		variant="ghost"
		size="sm"
		aria-label={$t.transcript.find.previous}
		disabled={!total}
		onclick={() => onStep(-1)}
	>
		<svg
			width="0.875rem"
			height="0.875rem"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
			aria-hidden="true"
		>
			<path d="m6 15 6-6 6 6" />
		</svg>
	</ToolButton>
	<ToolButton
		variant="ghost"
		size="sm"
		aria-label={$t.transcript.find.next}
		disabled={!total}
		onclick={() => onStep(1)}
	>
		<svg
			width="0.875rem"
			height="0.875rem"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
			aria-hidden="true"
		>
			<path d="m6 9 6 6 6-6" />
		</svg>
	</ToolButton>
	<ToolButton
		variant="ghost"
		size="sm"
		aria-label={$t.transcript.find.close}
		onclick={() => onClose(returnTo)}
	>
		<svg
			width="0.875rem"
			height="0.875rem"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			aria-hidden="true"
		>
			<path d="M6 6l12 12M18 6 6 18" />
		</svg>
	</ToolButton>
	<p class="sr-only" role="status">{countText}</p>
</div>

<style>
	.find {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
	}
	.field {
		flex: 1 1 12em;
		min-width: 0;
		font: inherit;
		font-size: var(--type-small);
		min-height: var(--control-sm);
		color: var(--text-body);
		background: var(--surface-0);
		border: 1px solid var(--line-strong);
		border-radius: var(--radius-control);
		padding: var(--space-1) var(--space-2);
	}
	/* The bar has its own Close; the search field's own clear button would be a second ×. */
	.field::-webkit-search-cancel-button {
		appearance: none;
	}
	.count {
		font-family: var(--font-mono);
		font-size: var(--type-caption);
		color: var(--text-muted);
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}
</style>
