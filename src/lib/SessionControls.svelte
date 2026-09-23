<script lang="ts">
	import { t } from './i18n';
	import ToolButton from './ui/ToolButton.svelte';
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
		<ToolButton
			variant="danger"
			size="lg"
			class="stop"
			disabled={busy}
			aria-busy={busy}
			onclick={onStop}
		>
			<svg width="13" height="13" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"
				><rect x="6" y="6" width="12" height="12" rx="2" /></svg
			>
			{busy ? $t.rail.stopping : $t.rail.stop}
		</ToolButton>
	{:else}
		<ToolButton
			variant="primary"
			size="lg"
			class="start"
			disabled={startDisabled}
			aria-busy={busy}
			onclick={onStart}
		>
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
		</ToolButton><ToolButton
			variant="ghost"
			size="lg"
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
		</ToolButton>
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
	.session-controls :global(.start),
	.session-controls :global(.stop) {
		min-width: min(15rem, 100%);
	}
	.start-key {
		align-self: center;
	}
</style>
