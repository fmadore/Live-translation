<script lang="ts">
	import type { Snippet } from 'svelte';
	import { t } from './i18n';
	import { isRunning, options, sessionStartedAt, sessionState } from './stores';
	import type { SessionState } from './types';

	/** The window's one bar: the session's actions on the left, its state and the settings on
	 *  the right. Windows already prints the app's name in the frame above it, so this bar spends
	 *  its width on the controls instead. */
	let {
		elapsed,
		settingsOpen,
		onOpenSettings,
		actions
	}: {
		/** The running session's clock, already formatted. */
		elapsed: string;
		settingsOpen: boolean;
		onOpenSettings: () => void;
		/** Start and Rehearse, or Stop: in the same bar as the status they change. */
		actions: Snippet;
	} = $props();

	// Not from the catalog: these are CSS class names, not words.
	const stateTone: Record<SessionState, string> = {
		idle: 'neutral',
		connecting: 'warn',
		running: 'live',
		reconnecting: 'warn',
		error: 'bad'
	};
</script>

<header class="toolbar">
	<span class="brand" aria-hidden="true">
		<svg
			width="14"
			height="14"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2.4"
			stroke-linecap="round"><path d="M4 12.5h3.5L11 6l3 12 2.5-5.5H20" /></svg
		>
	</span>
	<!-- The window's one h1, which every heading in either column sits under. The frame
	     already shows the name, so it is here for heading navigation only. -->
	<h1 class="sr-only">{$t.app.name}</h1>
	{@render actions()}
	<div class="pill {stateTone[$sessionState]}">
		<span class="pill-dot" aria-hidden="true"></span>
		<span class="pill-label"
			>{$sessionState === 'running' && $options.provider === 'ondevice'
				? $t.state.demo
				: $t.state[$sessionState]}</span
		>
		{#if $isRunning && $sessionStartedAt !== null}
			<span class="pill-time">{elapsed}</span>
		{/if}
	</div>
	<!-- Persistent access to caption, reading, history and app preferences. -->
	<button
		class="gear"
		aria-haspopup="dialog"
		aria-expanded={settingsOpen}
		aria-label={$t.settings.openLabel}
		onclick={onOpenSettings}
	>
		<svg
			width="15"
			height="15"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="1.7"
			stroke-linecap="round"
			stroke-linejoin="round"
			aria-hidden="true"
		>
			<circle cx="12" cy="12" r="3" />
			<path
				d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"
			/>
		</svg>
	</button>
</header>

<style>
	/* Sized by its content, not a slot: at 225% the buttons are twice as tall, and when the
	   bar no longer fits on one line it wraps, keeping the status and the gear at the right. */
	.toolbar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 0.75rem;
		padding: 0.625rem 1.375rem;
		background: var(--surface-1);
	}
	/* The one control in the bar whose position never depends on what the session is doing.
	   Icon-only, so its accessible name carries the whole label; the box is padded out to a
	   real target rather than left the size of the glyph. */
	.gear {
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 0 0 auto;
		width: 2rem;
		height: 2rem;
		border-radius: var(--radius-control);
		border: 1px solid transparent;
		background: transparent;
		color: var(--text-muted);
	}
	.gear:hover {
		border-color: var(--line-hover);
		color: var(--text-body);
	}
	.brand {
		width: 1.5rem;
		height: 1.5rem;
		border-radius: 6px;
		background: linear-gradient(150deg, var(--accent), var(--accent-deep));
		color: var(--on-accent);
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 0 0 auto;
	}
	/* Pushed to the far end, and kept there with the gear when the bar wraps. */
	.pill {
		display: flex;
		align-items: center;
		margin-left: auto;
		gap: 7px;
		padding: 4px 10px 4px 8px;
		border-radius: 20px;
		flex: 0 0 auto;
	}
	.pill-dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
	}
	.pill-label {
		font-size: var(--type-caption);
		font-weight: 500;
		line-height: 1;
		letter-spacing: 0.06em;
		text-transform: uppercase;
	}
	.pill-time {
		font-family: var(--font-mono);
		font-size: var(--type-caption);
		font-weight: 500;
		line-height: 1;
		font-variant-numeric: tabular-nums;
	}
	.pill.neutral {
		background: var(--surface-2);
		border: 1px solid var(--line-strong);
	}
	.pill.neutral .pill-dot {
		background: var(--faint);
	}
	.pill.neutral .pill-label {
		color: var(--text-muted);
	}
	.pill.live {
		background: var(--accent-bg);
		border: 1px solid var(--accent-border);
	}
	.pill.live .pill-dot {
		background: var(--accent);
		box-shadow: 0 0 8px var(--accent);
		animation: breathe 2.4s ease-in-out infinite;
	}
	.pill.live .pill-label,
	.pill.live .pill-time {
		color: var(--accent-soft);
		font-weight: 600;
	}
	.pill.warn {
		background: var(--warn-bg);
		border: 1px solid var(--warn-border);
	}
	.pill.warn .pill-dot {
		background: var(--warn);
		animation: breathe 2.4s ease-in-out infinite;
	}
	.pill.warn .pill-label {
		color: var(--warn-soft);
	}
	.pill.bad {
		background: var(--danger-bg);
		border: 1px solid var(--danger-border);
	}
	.pill.bad .pill-dot {
		background: var(--danger);
	}
	.pill.bad .pill-label {
		color: var(--danger-soft);
	}

	@media (prefers-reduced-motion: reduce) {
		.pill-dot {
			animation: none;
		}
	}

	/* Live/idle/error is a dot colour plus a word; only the word survives a contrast theme, so
	   the dot stops pretending to carry state and the pill keeps a visible edge. */
	@media (forced-colors: active) {
		.pill {
			border: 1px solid CanvasText;
		}
		.pill-dot {
			display: none;
		}
	}
</style>
