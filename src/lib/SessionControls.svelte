<script lang="ts">
	import { t } from './i18n';
	import ToolButton from './ui/ToolButton.svelte';
	import { ariaKeyShortcut, keyLabel } from './shortcuts';
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

	// The same key starts and stops, so both buttons carry it. Printed inside the button rather
	// than beside it, where it read as a third button; announced through `aria-keyshortcuts`,
	// so the printed copy stays out of the accessible name.
	const keys = $derived(keyLabel('toggleSession', $t.keys));
	const ariaKeys = ariaKeyShortcut('toggleSession');
</script>

<!-- Start and Stop live in the window's one bar, so they stay in the same place while setup,
     captions and transcripts scroll underneath. -->
<div class="session-controls">
	{#if $isRunning}
		<ToolButton
			variant="danger"
			size="lg"
			class="stop"
			disabled={busy}
			aria-busy={busy}
			aria-keyshortcuts={ariaKeys}
			onclick={onStop}
		>
			<svg width="13" height="13" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"
				><rect x="6" y="6" width="12" height="12" rx="2" /></svg
			>
			{busy ? $t.rail.stopping : $t.rail.stop}
			<span class="shortcut" aria-hidden="true">{keys}</span>
		</ToolButton>
	{:else}
		<ToolButton
			variant="primary"
			size="lg"
			class="start"
			disabled={startDisabled}
			aria-busy={busy}
			aria-keyshortcuts={ariaKeys}
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
			<span class="shortcut" aria-hidden="true">{keys}</span>
		</ToolButton>
		<ToolButton
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
</div>

<style>
	/* The buttons are items of the bar itself, so a narrow bar wraps between them rather than
	   around the pair. */
	.session-controls {
		display: contents;
	}
	/* The key as secondary text: the button's own colour and face, set off by a hairline of
	   that colour so it reads as a note on the action rather than part of its name. */
	.shortcut {
		margin-left: 0.25rem;
		padding-left: 0.625rem;
		border-left: 1px solid color-mix(in srgb, currentColor 35%, transparent);
		font-family: var(--font-mono);
		font-size: var(--type-caption);
		font-weight: 500;
		white-space: nowrap;
	}
	/* Once the bar can no longer hold it on one line — a narrow window, or a large Windows text
	   size, since the query is in `em` — the printed key goes and the buttons keep their room.
	   It is still announced, and still listed under Settings → App → Keyboard shortcuts. */
	@container window (max-width: 48em) {
		.shortcut {
			display: none;
		}
	}
</style>
