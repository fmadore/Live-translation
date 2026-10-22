<script lang="ts">
	import { tick, untrack } from 'svelte';
	import { t } from './i18n';
	import ToolButton from './ui/ToolButton.svelte';
	import { ariaKeyShortcut, keyLabel } from './shortcuts';
	import { isRunning, options } from './stores';

	let {
		busy,
		startDisabled,
		rehearseDisabled,
		paused,
		onStart,
		onRehearse,
		onStop,
		onPause
	}: {
		/** A start or stop is in flight. */
		busy: boolean;
		startDisabled: boolean;
		rehearseDisabled: boolean;
		/** The running session is paused. */
		paused: boolean;
		onStart: () => void;
		onRehearse: () => void;
		onStop: () => void;
		onPause: () => void;
	} = $props();

	// The same key starts and stops, so both buttons carry it. Printed inside the button rather
	// than beside it, where it read as a third button; announced through `aria-keyshortcuts`,
	// so the printed copy stays out of the accessible name.
	const keys = $derived(keyLabel('toggleSession', $t.keys));
	const ariaKeys = ariaKeyShortcut('toggleSession');
	const pauseKeys = $derived(keyLabel('togglePause', $t.keys));

	let startEl = $state<HTMLButtonElement | null>(null);
	let rehearseEl = $state<HTMLButtonElement | null>(null);
	let stopEl = $state<HTMLButtonElement | null>(null);
	let pauseEl = $state<HTMLButtonElement | null>(null);

	// Start and Rehearse make way for Stop and Pause as a run begins, and the other way round
	// as it ends, so the button the keyboard was on is removed and focus falls to the page.
	// Noted before the swap reaches the DOM, and handed to the counterpart once it is there:
	// Stop after a start, Start after a stop. Only when focus was on one of these, so a run
	// ending by itself never pulls the keyboard out of the setup or a transcript.
	$effect.pre(() => {
		const running = $isRunning;
		const held = untrack(() => [startEl, rehearseEl, stopEl, pauseEl]);
		if (!held.some((button) => button !== null && button === document.activeElement)) return;
		void tick().then(() => (running ? stopEl : startEl)?.focus());
	});
</script>

<!-- Start and Stop live in the window's one bar, so they stay in the same place while setup,
     captions and transcripts scroll underneath.

     While a start or stop is under way, Start, Rehearse and Stop are `aria-disabled` rather
     than `disabled`: a disabled button drops the focus it holds, and the keyboard would be gone
     before the swap above could hand it on. A press is ignored instead, and the style below
     draws them as disabled. -->
<div class="session-controls">
	{#if $isRunning}
		<ToolButton
			variant="danger"
			size="lg"
			class="stop"
			bind:element={stopEl}
			aria-disabled={busy || undefined}
			aria-busy={busy}
			aria-keyshortcuts={ariaKeys}
			onclick={() => {
				if (!busy) onStop();
			}}
		>
			<svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"
				><rect x="6" y="6" width="12" height="12" rx="2" /></svg
			>
			{busy ? $t.rail.stopping : $t.rail.stop}
			<span class="shortcut" aria-hidden="true">{keys}</span>
		</ToolButton>
		<!-- Pausing keeps the session, its clock and its transcript: only the caption engine is
		     let go, so a coffee break or a video clip is not billed. The label says what a
		     press does, Pause or Resume, so it is not also a toggle: Narrator read a pressed
		     "Resume" as "Resume, toggle button, pressed". -->
		<ToolButton
			variant={paused ? 'primary' : 'ghost'}
			size="lg"
			bind:element={pauseEl}
			disabled={busy}
			aria-keyshortcuts={ariaKeyShortcut('togglePause')}
			onclick={onPause}
		>
			{#if paused}
				<svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"
					><path d="M8 5.5l11 6.5-11 6.5z" /></svg
				>
			{:else}
				<svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"
					><rect x="6" y="5" width="4" height="14" rx="1.2" /><rect
						x="14"
						y="5"
						width="4"
						height="14"
						rx="1.2"
					/></svg
				>
			{/if}
			{paused ? $t.rail.resume : $t.rail.pause}
			<span class="shortcut" aria-hidden="true">{pauseKeys}</span>
		</ToolButton>
	{:else}
		<ToolButton
			variant="primary"
			size="lg"
			class="start"
			bind:element={startEl}
			disabled={startDisabled && !busy}
			aria-disabled={busy || undefined}
			aria-busy={busy}
			aria-keyshortcuts={ariaKeys}
			onclick={() => {
				if (!busy) onStart();
			}}
		>
			<svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"
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
			bind:element={rehearseEl}
			disabled={rehearseDisabled && !busy}
			aria-disabled={busy || undefined}
			aria-busy={busy}
			onclick={() => {
				if (!busy) onRehearse();
			}}
		>
			<svg
				width="14"
				height="14"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.75"
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
	/* A busy session action, kept focusable (see the markup), looks as `app.css` draws a
	   disabled button and takes no pointer: no hover, and a click lands on the bar. */
	.session-controls :global(.ui-tool[aria-disabled='true']) {
		opacity: 0.5;
		pointer-events: none;
	}
	/* The key as secondary text: the button's own colour and face, set off by a hairline of
	   that colour so it reads as a note on the action rather than part of its name. */
	.shortcut {
		margin-left: var(--space-1);
		padding-left: var(--space-2);
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
	/* A contrast theme greys a `disabled` button by itself, but not an `aria-disabled` one. */
	@media (forced-colors: active) {
		.session-controls :global(.ui-tool[aria-disabled='true']) {
			opacity: 1;
			border-color: GrayText;
			color: GrayText;
		}
	}
</style>
