<script lang="ts">
	import { t } from './i18n';
	import { isRunning, options } from './stores';

	let {
		busy,
		startDisabled,
		rehearseDisabled,
		onStart,
		onRehearse,
		onStop
	}: {
		/** A start or stop is in flight. */
		busy: boolean;
		startDisabled: boolean;
		rehearseDisabled: boolean;
		onStart: () => void;
		onRehearse: () => void;
		onStop: () => void;
	} = $props();
</script>

<!-- Start and Stop share one persistent bar below the header, so they stay in the same place
     while setup, captions and transcripts scroll underneath. -->
<div class="session-controls">
	{#if $isRunning}
		<button class="stop" disabled={busy} aria-busy={busy} onclick={onStop}>
			<svg width="13" height="13" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"
				><rect x="6" y="6" width="12" height="12" rx="2" /></svg
			>
			{busy ? $t.rail.stopping : $t.rail.stop}
		</button>
	{:else}
		<button class="start" disabled={startDisabled} aria-busy={busy} onclick={onStart}>
			<svg width="15" height="15" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"
				><path d="M8 5.5l11 6.5-11 6.5z" /></svg
			>
			{busy
				? $t.preflight.start.starting
				: $options.mode === 'translate'
					? $t.preflight.start.translate
					: $options.provider === 'ondevice'
						? $t.preflight.start.demo
						: $t.preflight.start.subtitles}
		</button><button
			class="rehearse"
			aria-describedby="rehearse-hint"
			disabled={rehearseDisabled}
			onclick={onRehearse}
		>
			<svg
				width="13"
				height="13"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.8"
				stroke-linecap="round"
				stroke-linejoin="round"
				aria-hidden="true"
				><path d="M4 9.5h3.5L13 5v14L7.5 14.5H4z" /><path d="M17 8.5l3.5 3.5-3.5 3.5" /></svg
			>
			{busy ? $t.preflight.start.starting : $t.preflight.rehearse.action}
		</button>
	{/if}
	<span class="key start-key" aria-hidden="true">Ctrl Shift Space</span>
</div>

<style>
	.session-controls {
		padding: 0.75rem 1.375rem;
		border-bottom: 1px solid var(--line-strong);
		background: var(--surface-1);
		display: flex;
		flex-wrap: wrap;
		align-items: stretch;
		gap: 12px;
	}
	.session-controls .start,
	.session-controls .stop {
		min-width: min(15rem, 100%);
		justify-content: center;
	}
	.start-key {
		align-self: center;
	}
	.start {
		display: flex;
		align-items: center;
		gap: 11px;
		padding: 15px 24px;
		border: 0;
		border-radius: var(--radius-card);
		background: linear-gradient(#5ad1a0, #43b989);
		color: var(--on-accent);
		font-size: var(--type-label);
		font-weight: 600;
		line-height: 1;
		box-shadow: 0 12px 30px -12px rgba(90, 209, 160, 0.65);
		flex: 0 0 auto;
	}
	.start:hover:not(:disabled) {
		filter: brightness(1.06);
	}
	.start:disabled {
		box-shadow: none;
	}
	.stop {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 10px;
		padding: 14px;
		border-radius: var(--radius-card);
		border: 1px solid var(--danger-border);
		background: var(--danger-bg);
		color: var(--danger-soft);
		font-size: var(--type-label);
		font-weight: 600;
		line-height: 1;
		flex: 0 0 auto;
	}
	.stop:hover:not(:disabled) {
		background: rgba(255, 92, 92, 0.18);
		color: #ffb3b3;
	}
	/* Quiet companion to Start: same row, none of the weight — a rehearsal is a dry run, not
	   the thing the operator came to press. */
	.rehearse {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 13px 18px;
		border-radius: var(--radius-card);
		border: 1px solid var(--line-strong);
		background: transparent;
		color: var(--text-secondary);
		font-size: var(--type-body);
		font-weight: 500;
		line-height: 1;
		white-space: nowrap;
		flex: 0 0 auto;
	}
	.rehearse:hover:not(:disabled) {
		border-color: var(--line-hover);
		color: var(--text-body);
	}

	/* Gradients are not recoloured by the forced palette, so the button would keep its mint
	   fill under system-coloured text. Drop it and let the theme paint the button. */
	@media (forced-colors: active) {
		.start {
			background-image: none;
		}
	}
</style>
