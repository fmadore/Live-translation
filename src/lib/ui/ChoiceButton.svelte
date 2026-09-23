<script lang="ts">
	import type { Snippet } from 'svelte';

	/** One of a set of mutually exclusive choices, drawn as a card with an icon, a title and a
	 *  sentence, and a tick on the chosen one. */
	let {
		selected,
		disabled = false,
		title,
		desc,
		icon,
		onclick
	}: {
		selected: boolean;
		disabled?: boolean;
		title: string;
		desc: string;
		icon: Snippet;
		onclick: () => void;
	} = $props();
</script>

<button class="card" class:selected {disabled} aria-pressed={selected} {onclick}>
	<span class="card-icon" aria-hidden="true">{@render icon()}</span>
	<span class="card-body">
		<span class="card-title">{title}</span>
		<span class="card-desc">{desc}</span>
	</span>
	{#if selected}
		<span class="card-check" aria-hidden="true">
			<svg
				width="16"
				height="16"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2.2"
				stroke-linecap="round"><path d="M4 12.5l5 5L20 6.5" /></svg
			>
		</span>
	{/if}
</button>

<style>
	.card {
		display: flex;
		align-items: flex-start;
		gap: 0.6875rem;
		padding: 0.75rem 0.8125rem;
		border: 1px solid var(--line-strong);
		border-radius: var(--radius-card);
		background: var(--surface-1);
		width: 100%;
		text-align: left;
		color: inherit;
	}
	.card.selected {
		border-color: var(--accent-border);
		background: var(--accent-bg);
	}
	.card:hover:not(:disabled) {
		border-color: var(--line-hover);
	}
	.card.selected:hover:not(:disabled) {
		border-color: var(--accent);
	}
	.card-icon {
		width: 30px;
		height: 30px;
		border-radius: var(--radius-control);
		background: rgba(255, 255, 255, 0.045);
		color: var(--text-muted);
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 0 0 auto;
	}
	.card.selected .card-icon {
		background: var(--accent-chip-bg);
		color: var(--accent-soft);
	}
	.card-body {
		display: flex;
		flex-direction: column;
		gap: 3px;
		min-width: 0;
	}
	.card-title {
		font-size: var(--type-label);
		font-weight: 600;
		line-height: 1.2;
		color: var(--text-secondary);
	}
	.card.selected .card-title {
		color: var(--text-body);
	}
	.card-desc {
		font-size: var(--type-small);
		line-height: 1.45;
		color: var(--text-muted);
		text-wrap: pretty;
	}
	.card-check {
		color: var(--accent);
		margin-left: auto;
		flex: 0 0 auto;
		display: flex;
	}
	/* Selection is a mint border and a mint wash, and both flatten to the same Canvas and
	   CanvasText as the card next to it in a contrast theme. An inset outline survives. */
	@media (forced-colors: active) {
		.card.selected {
			outline: 2px solid Highlight;
			outline-offset: -2px;
		}
	}
</style>
