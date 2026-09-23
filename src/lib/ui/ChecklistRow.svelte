<script lang="ts">
	import type { Snippet } from 'svelte';

	/**
	 * One row of the pre-flight checklist: a mark saying whether the check has passed, a
	 * title and a sentence, and an optional action at the end of the row.
	 */
	let {
		status,
		glyph = '',
		title,
		titleFor,
		desc,
		descId,
		warn = false,
		pending = false,
		description,
		action
	}: {
		/** `ok` draws a tick, `wait` an amber dot, `neutral` the `glyph`. */
		status: 'ok' | 'wait' | 'neutral';
		glyph?: string;
		title: string;
		/** Render the title as the label of this field. */
		titleFor?: string;
		desc?: string;
		descId?: string;
		/** The sentence describes something that still needs doing. */
		warn?: boolean;
		/** The row holds a form that wraps: align to the top rather than a shifting centre. */
		pending?: boolean;
		/** Rich content in place of `desc`. */
		description?: Snippet;
		action?: Snippet;
	} = $props();
</script>

<div class="check-row" class:pending>
	{#if status === 'ok'}
		<span class="mark ok" aria-hidden="true">
			<svg
				width="12"
				height="12"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2.6"
				stroke-linecap="round"
				aria-hidden="true"><path d="M4 12.5l5 5L20 6.5" /></svg
			>
		</span>
	{:else if status === 'wait'}
		<span class="mark wait" aria-hidden="true"><span class="dot"></span></span>
	{:else}
		<span class="mark neutral" aria-hidden="true">{glyph}</span>
	{/if}
	<div class="check-body">
		{#if titleFor}
			<label class="check-title" for={titleFor}>{title}</label>
		{:else}
			<span class="check-title">{title}</span>
		{/if}
		<span class="check-desc" class:warn id={descId}>
			{#if description}{@render description()}{:else}{desc}{/if}
		</span>
	</div>
	{#if action}{@render action()}{:else}<span></span>{/if}
</div>

<style>
	.check-row {
		display: grid;
		grid-template-columns: 24px 1fr auto;
		align-items: center;
		gap: 14px;
		padding: 15px 0;
		border-bottom: 1px solid var(--hairline);
	}
	.check-row.pending {
		align-items: start;
	}
	.mark {
		width: 20px;
		height: 20px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.check-row.pending .mark {
		margin-top: 3px;
	}
	.mark.ok {
		background: var(--accent-chip-bg);
		color: var(--accent);
	}
	.mark.wait {
		background: var(--warn-bg);
		color: var(--warn);
	}
	.mark.wait .dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: currentColor;
	}
	.mark.neutral {
		background: rgba(255, 255, 255, 0.05);
		color: var(--muted);
		font-family: var(--font-mono);
		font-size: var(--type-11);
		font-weight: 500;
		line-height: 1;
	}
	.check-body {
		display: flex;
		flex-direction: column;
		gap: 3px;
		min-width: 0;
	}
	.check-title {
		font-size: var(--type-13-5);
		font-weight: 500;
		line-height: 1.2;
	}
	.check-desc {
		font-size: var(--type-12);
		line-height: 1.3;
		color: var(--muted-2);
	}
	/* A pending row's sentence wraps around a link and a model id. */
	.check-row.pending .check-desc {
		line-height: 1.35;
	}
	.check-desc.warn {
		color: var(--warn);
	}
	/* The tighter checklist rhythm the stage adopts near the window's minimum height. */
	@media (max-height: 740px) {
		.check-row {
			padding: 12px 0;
		}
	}
</style>
