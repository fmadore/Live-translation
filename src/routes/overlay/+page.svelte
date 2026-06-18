<script lang="ts">
	import { onMount } from 'svelte';
	import { on, isTauri } from '$lib/tauri';
	import type { Caption } from '$lib/types';
	import { DEFAULT_OVERLAY_FONT, OVERLAY_FONT_KEY } from '$lib/types';

	// The overlay keeps only what it needs to render: the current line(s).
	let current = $state<Caption | null>(null);
	let previous = $state<string>('');

	// Initial size comes from the shared localStorage key (same origin as the operator),
	// then the operator pushes live updates via the overlay-config event.
	function initialFont(): number {
		if (typeof localStorage === 'undefined') return DEFAULT_OVERLAY_FONT;
		const v = Number(localStorage.getItem(OVERLAY_FONT_KEY));
		return Number.isFinite(v) && v > 0 ? v : DEFAULT_OVERLAY_FONT;
	}
	let fontSize = $state(initialFont());

	// Auto-hide captions after the last update so the overlay never sits on a stale line
	// over the slides. A finalized line gets a short reading pause; an in-progress line
	// that stalls (Gemini didn't send `turnComplete`) clears a little sooner.
	const FINAL_HOLD_MS = 4000;
	const INTERIM_HOLD_MS = 3000;
	let clearTimer: ReturnType<typeof setTimeout> | undefined;

	onMount(() => {
		if (!isTauri()) {
			// Demo content so the overlay can be previewed in a browser.
			current = {
				turnId: 0,
				text: 'Bienvenue — les sous-titres apparaîtront ici.',
				sourceText: '',
				final: true,
				origin: 'system'
			};
			return;
		}

		const unlistenCaption = on.caption((c) => {
			if (c.final && current && current.turnId !== c.turnId && current.text.trim()) {
				previous = current.text;
			}
			current = c;
			// Always re-arm the auto-hide: even when a turn ends on an interim update and
			// Gemini never sends `turnComplete`, the line must still disappear on its own.
			clearTimeout(clearTimer);
			clearTimer = setTimeout(
				() => {
					previous = '';
					current = null;
				},
				c.final ? FINAL_HOLD_MS : INTERIM_HOLD_MS
			);
		});

		const unlistenConfig = on.overlayConfig((cfg) => {
			if (Number.isFinite(cfg.fontSize) && cfg.fontSize > 0) fontSize = cfg.fontSize;
		});

		return () => {
			void unlistenCaption.then((f) => f());
			void unlistenConfig.then((f) => f());
		};
	});

	function bump(delta: number) {
		fontSize = Math.max(20, Math.min(96, fontSize + delta));
	}
</script>

<svelte:window
	onkeydown={(e) => {
		if (e.key === '+' || e.key === '=') bump(2);
		if (e.key === '-') bump(-2);
	}}
/>

<div class="stage">
	<div class="captions" style="--fs: {fontSize}px">
		{#if previous}
			<p class="line prev">{previous}</p>
		{/if}
		{#if current}
			<p class="line cur" class:interim={!current.final}>{current.text}</p>
		{/if}
	</div>
</div>

<style>
	/* The window itself is transparent (Tauri transparent:true); only the caption
	   backing is painted so text stays legible over any slide. */
	:global(html),
	:global(body) {
		background: transparent !important;
	}
	.stage {
		position: fixed;
		inset: 0;
		display: flex;
		align-items: flex-end;
		justify-content: center;
		padding: 0 6vw 6vh;
		pointer-events: none;
		user-select: none;
	}
	.captions {
		max-width: 90%;
		text-align: center;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.line {
		margin: 0;
		font-size: var(--fs);
		line-height: 1.25;
		font-weight: 700;
		color: #ffffff;
		background: rgba(0, 0, 0, 0.62);
		padding: 10px 22px;
		border-radius: 12px;
		/* Heavy text shadow keeps it readable even where the backing is thin. */
		text-shadow:
			0 2px 4px rgba(0, 0, 0, 0.9),
			0 0 2px rgba(0, 0, 0, 0.9);
		backdrop-filter: blur(2px);
	}
	.line.prev {
		font-size: calc(var(--fs) * 0.62);
		opacity: 0.6;
		font-weight: 600;
	}
	.line.interim {
		opacity: 0.82;
	}
</style>
