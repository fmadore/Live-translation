<script lang="ts">
	import { onMount } from 'svelte';
	import { on, isTauri } from '$lib/tauri';
	import type { Caption, Origin } from '$lib/types';
	import { clampOverlayFont, loadOverlayFont } from '$lib/types';

	// The overlay keeps only what it needs to render: one current line per origin (mic and
	// system turns have independent ids, so they must never share a slot) plus the last
	// finalized line, shown dimmed while a new turn streams in.
	let current = $state<Partial<Record<Origin, Caption>>>({});
	let previous = $state<string>('');

	// Stable render order: the remote speaker (system) above the room mic.
	const ORIGIN_ORDER: Origin[] = ['system', 'microphone'];
	const lines = $derived(
		ORIGIN_ORDER.flatMap((o) => (current[o] ? [current[o] as Caption] : []))
	);

	// Initial size comes from the shared localStorage key (same origin as the operator),
	// then the operator pushes live updates via the overlay-config event.
	let fontSize = $state(loadOverlayFont());

	// Move mode (operator-driven): click-through is off and the whole stage becomes a
	// Tauri drag region so the window can be dragged/resized into place.
	let interactive = $state(false);

	// Auto-hide captions after the last update so the overlay never sits on a stale line
	// over the slides. A finalized line gets a short reading pause; an in-progress line
	// that stalls (no turn-complete arriving) clears a little sooner.
	const FINAL_HOLD_MS = 4000;
	const INTERIM_HOLD_MS = 3000;
	const clearTimers: Partial<Record<Origin, ReturnType<typeof setTimeout>>> = {};

	// Keep captions subtitle-sized. A turn streams until it completes, which during
	// continuous speech can run for many sentences, so render only the most recent slice of
	// the (still-growing) turn instead of the whole thing — otherwise it fills the screen.
	const MAX_CHARS = 220;
	function tail(text: string): string {
		const t = text.replace(/\s+/g, ' ').trim();
		if (t.length <= MAX_CHARS) return t;
		let cut = t.length - MAX_CHARS;
		// Don't start mid-word: jump to the next space if it's close.
		const sp = t.indexOf(' ', cut);
		if (sp !== -1 && sp - cut < 24) cut = sp + 1;
		return '… ' + t.slice(cut);
	}

	onMount(() => {
		if (!isTauri()) {
			// Demo content so the overlay can be previewed in a browser.
			current.system = {
				turnId: 0,
				text: 'Bienvenue — les sous-titres apparaîtront ici.',
				sourceText: '',
				final: true,
				origin: 'system'
			};
			return;
		}

		const unlistenCaption = on.caption((c) => {
			const cur = current[c.origin];
			// A caption for a new turn of this origin: keep the finished line dimmed above
			// the fresh one while it streams in.
			if (cur && cur.final && cur.turnId !== c.turnId && cur.text.trim()) {
				previous = cur.text;
			}
			current[c.origin] = c;
			// Always re-arm this origin's auto-hide: even when a turn ends on an interim
			// update and no turn-complete ever arrives, the line must disappear on its own.
			clearTimeout(clearTimers[c.origin]);
			clearTimers[c.origin] = setTimeout(
				() => {
					delete current[c.origin];
					if (!ORIGIN_ORDER.some((o) => current[o])) previous = '';
				},
				c.final ? FINAL_HOLD_MS : INTERIM_HOLD_MS
			);
		});

		const unlistenConfig = on.overlayConfig((cfg) => {
			if (Number.isFinite(cfg.fontSize) && cfg.fontSize > 0)
				fontSize = clampOverlayFont(cfg.fontSize);
			if (typeof cfg.interactive === 'boolean') interactive = cfg.interactive;
		});

		return () => {
			void unlistenCaption.then((f) => f());
			void unlistenConfig.then((f) => f());
		};
	});

	// Reachable while move mode has the window focused (it's click-through otherwise).
	function bump(delta: number) {
		fontSize = clampOverlayFont(fontSize + delta);
	}
</script>

<svelte:window
	onkeydown={(e) => {
		if (e.key === '+' || e.key === '=') bump(2);
		if (e.key === '-') bump(-2);
	}}
/>

<div class="stage" class:interactive data-tauri-drag-region={interactive || undefined}>
	{#if interactive}
		<p class="move-hint">Drag to move · drag edges to resize · “Done moving” in the operator window locks it</p>
	{/if}
	<div class="captions" style="--fs: {fontSize}px">
		{#if previous}
			<p class="line prev">{tail(previous)}</p>
		{/if}
		{#each lines as line (line.origin)}
			<p class="line cur" class:interim={!line.final}>{tail(line.text)}</p>
		{/each}
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
	.stage.interactive {
		pointer-events: auto;
		cursor: move;
		outline: 2px dashed rgba(255, 255, 255, 0.55);
		outline-offset: -3px;
		background: rgba(30, 60, 120, 0.12);
	}
	.move-hint {
		position: absolute;
		top: 10px;
		left: 50%;
		transform: translateX(-50%);
		margin: 0;
		font-size: 14px;
		color: #fff;
		background: rgba(0, 0, 0, 0.62);
		padding: 6px 14px;
		border-radius: 8px;
		pointer-events: none;
		white-space: nowrap;
	}
	.captions {
		max-width: 90%;
		text-align: center;
		display: flex;
		flex-direction: column;
		gap: 8px;
		pointer-events: none;
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
