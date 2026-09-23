<script lang="ts">
	import { t } from './i18n';
	import { isRunning, options, sessionStartedAt, sessionState } from './stores';
	import type { SessionState } from './types';

	let {
		elapsed,
		settingsOpen,
		onOpenSettings
	}: {
		/** The running session's clock, already formatted. */
		elapsed: string;
		settingsOpen: boolean;
		onOpenSettings: () => void;
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

<header class="titlebar">
	<span class="brand" aria-hidden="true">
		<svg
			width="11"
			height="11"
			viewBox="0 0 24 24"
			fill="none"
			stroke="#06261b"
			stroke-width="2.4"
			stroke-linecap="round"><path d="M4 12.5h3.5L11 6l3 12 2.5-5.5H20" /></svg
		>
	</span>
	<h1 class="app-name">{$t.app.name}</h1>
	<span class="context">{$t.app.tagline}</span>
	<span class="grow"></span>
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
	.titlebar {
		display: flex;
		align-items: center;
		gap: 10px;
		min-height: 40px;
		padding: 4px 14px;
		background: var(--surface-1);
		border-bottom: 1px solid var(--line);
	}
	/* The only control in the titlebar, and the only one whose position never depends on what
	   the session is doing. Icon-only, so its accessible name carries the whole label; the box
	   is padded out to a real target rather than left the size of the glyph. */
	.gear {
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 0 0 auto;
		width: 28px;
		height: 28px;
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
		width: 18px;
		height: 18px;
		border-radius: 5px;
		background: linear-gradient(150deg, #5ad1a0, #2f8f6b);
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 0 0 auto;
	}
	/* The window's one h1: every other heading in either column sits under it. */
	.app-name {
		margin: 0;
		font-size: var(--type-body);
		font-weight: 500;
		line-height: 1;
	}
	.context {
		font-size: var(--type-small);
		line-height: 1;
		color: var(--text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.pill {
		display: flex;
		align-items: center;
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
