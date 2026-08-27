<script lang="ts">
	import { onMount } from 'svelte';
	import { api, on, isTauri } from '$lib/tauri';
	import { locale, t } from '$lib/i18n';
	import type { Caption, Origin } from '$lib/types';
	import {
		captionBudget,
		clampOverlayFont,
		clampOverlayWidth,
		loadOverlayFont,
		loadOverlayWidth,
		OVERLAY_PLACED_KEY
	} from '$lib/types';

	// The overlay keeps only what it needs to render: one current turn per origin (mic and
	// system turns have independent ids, so they must never share a slot) plus that origin's
	// previous turn, which trails into the current one so the audience can finish reading it.
	let current = $state<Partial<Record<Origin, Caption>>>({});
	let previous = $state<Partial<Record<Origin, string>>>({});

	// Stable render order: the remote speaker (system) above the room mic.
	const ORIGIN_ORDER: Origin[] = ['system', 'microphone'];

	// Words, not emoji or colour: at projector distance a two-letter cue is unreadable.
	const originLabel = $derived<Record<Origin, string>>($t.overlay.origin);

	// Initial size comes from the shared localStorage key (same origin as the operator),
	// then the operator pushes live updates via the overlay-config event.
	let fontSize = $state(loadOverlayFont());
	let captionWidth = $state(loadOverlayWidth());

	// Move mode (operator-driven): click-through is off and the whole stage becomes a
	// Tauri drag region so the window can be dragged/resized into place.
	let interactive = $state(false);

	// Live window size for the move-mode readout. The overlay window *is* the caption
	// region, so its own viewport is the number the operator needs.
	let winW = $state(0);
	let winH = $state(0);

	// Where the window sat when move mode was entered, so Escape can undo the whole move.
	// Physical pixels, straight off the window: restoring what was read needs no conversion.
	// Not reactive — nothing renders it.
	let entryGeometry: { x: number; y: number; width: number; height: number } | null = null;

	// Auto-hide captions after the last update so the overlay never sits on a stale line
	// over the slides. A finalized line gets a short reading pause; an in-progress line
	// that stalls (no turn-complete arriving) clears a little sooner.
	const FINAL_HOLD_MS = 4000;
	const INTERIM_HOLD_MS = 3000;
	const clearTimers: Partial<Record<Origin, ReturnType<typeof setTimeout>>> = {};

	// Keep captions subtitle-sized. A turn streams until it completes, which during
	// continuous speech can run for many sentences, so render only the most recent slice of
	// the (still-growing) turn instead of the whole thing — otherwise it fills the screen.
	// The budget follows the measure, so the block stays the same number of lines however
	// wide the operator sets the caption; see `captionBudget`.
	const maxChars = $derived(captionBudget(captionWidth));

	/** Keep the last `limit` characters, cutting on a word boundary. */
	function tail(text: string, limit: number): string {
		const t = text.replace(/\s+/g, ' ').trim();
		if (t.length <= limit) return t;
		let cut = t.length - limit;
		// Don't start mid-word: jump to the next space if it's close.
		const sp = t.indexOf(' ', cut);
		if (sp !== -1 && sp - cut < 24) cut = sp + 1;
		return '… ' + t.slice(cut);
	}

	// A speaker's previous turn and current turn are usually one continuous sentence, so they
	// render as one block of running text at one size: the tail of the finished turn (dimmed)
	// flowing straight into the live one. They share the character budget, so the block never
	// grows past a caption's worth — as the new turn streams in, the old text is pushed out.
	// Only the space left over by the current turn is spent on the lead-in, and once the
	// current turn fills the budget the previous one is gone entirely.
	// Fixed, unlike the budget above: this asks "is the leftover room worth anything to a
	// reader", and a readable fragment of a previous sentence is the same amount of text in a
	// narrow region as in a wide one.
	const MIN_LEAD_CHARS = 40;

	const lines = $derived(
		ORIGIN_ORDER.flatMap((origin) => {
			const caption = current[origin];
			if (!caption) return [];
			const text = tail(caption.text, maxChars);
			const room = maxChars - text.length;
			const lead = room >= MIN_LEAD_CHARS ? tail(previous[origin] ?? '', room) : '';
			return [{ origin, lead, text, interim: !caption.final }];
		})
	);

	// A single speaker needs no label — the row is unambiguous, and the label would only
	// steal width from the caption. Labels appear exactly when both origins are on screen.
	const showLabels = $derived(lines.length > 1);

	onMount(() => {
		const measure = () => {
			winW = Math.round(window.innerWidth);
			winH = Math.round(window.innerHeight);
		};
		measure();
		window.addEventListener('resize', measure);

		if (!isTauri()) {
			// Demo content so the overlay can be previewed in a browser: both origins visible
			// (so the labels show), one finalized line and one live turn carrying a lead-in.
			previous.system = 'Bienvenue — les sous-titres apparaîtront ici. Il me semble';
			current.system = {
				turnId: 1,
				text: "que c'est un peu bizarre comment le texte apparaît.",
				sourceText: '',
				final: false,
				origin: 'system'
			};
			current.microphone = {
				turnId: 1,
				text: 'So the corpus is about forty thousand documents.',
				sourceText: '',
				final: true,
				origin: 'microphone'
			};
			return () => window.removeEventListener('resize', measure);
		}

		const unlistenCaption = on.caption((c) => {
			const cur = current[c.origin];
			// A caption for a new turn of this origin: keep the finished text as the dimmed
			// lead-in to the fresh one. A turn can end without ever being flagged final, so
			// this keys off the turn id changing rather than on `cur.final`.
			if (cur && cur.turnId !== c.turnId && cur.text.trim()) {
				previous[c.origin] = cur.text;
			}
			current[c.origin] = c;
			// Always re-arm this origin's auto-hide: even when a turn ends on an interim
			// update and no turn-complete ever arrives, the line must disappear on its own.
			clearTimeout(clearTimers[c.origin]);
			clearTimers[c.origin] = setTimeout(
				() => {
					delete current[c.origin];
					delete previous[c.origin];
				},
				c.final ? FINAL_HOLD_MS : INTERIM_HOLD_MS
			);
		});

		const unlistenConfig = on.overlayConfig((cfg) => {
			if (Number.isFinite(cfg.fontSize) && cfg.fontSize > 0)
				fontSize = clampOverlayFont(cfg.fontSize);
			if (Number.isFinite(cfg.captionWidth) && (cfg.captionWidth ?? 0) > 0)
				captionWidth = clampOverlayWidth(cfg.captionWidth as number);
			// The operator owns the interface language; this window follows it.
			if (cfg.locale === 'en' || cfg.locale === 'fr') locale.set(cfg.locale);
			if (typeof cfg.interactive === 'boolean') {
				// Entering move mode: record the rect first, so Escape has something to restore.
				if (cfg.interactive && !interactive) void snapshotGeometry();
				interactive = cfg.interactive;
			}
		});

		return () => {
			window.removeEventListener('resize', measure);
			void unlistenCaption.then((f) => f());
			void unlistenConfig.then((f) => f());
		};
	});

	// Reachable while move mode has the window focused (it's click-through otherwise).
	function bump(delta: number) {
		fontSize = clampOverlayFont(fontSize + delta);
		// The operator window owns the size control too; tell it what happened here so the
		// two readouts never disagree. Fire-and-forget — a dropped event costs nothing.
		void api.emitOverlayState({ fontSize });
	}

	/** Remember the current window rect so a cancelled move can be undone. */
	async function snapshotGeometry() {
		if (!isTauri()) {
			entryGeometry = null;
			return;
		}
		try {
			const { getCurrentWindow } = await import('@tauri-apps/api/window');
			const win = getCurrentWindow();
			// `setPosition` takes the outer position and `setSize` the inner size, so read the
			// pair that can be handed straight back to them.
			const pos = await win.outerPosition();
			const size = await win.innerSize();
			entryGeometry = { x: pos.x, y: pos.y, width: size.width, height: size.height };
		} catch (err) {
			console.error('Could not record the overlay geometry', err);
			entryGeometry = null;
		}
	}

	/** Arrow-key nudge, in whole physical pixels — finer than a mouse drag can manage. */
	async function nudge(dx: number, dy: number) {
		if (!isTauri()) return;
		try {
			const { getCurrentWindow, PhysicalPosition } = await import('@tauri-apps/api/window');
			const win = getCurrentWindow();
			const pos = await win.outerPosition();
			await win.setPosition(new PhysicalPosition(pos.x + dx, pos.y + dy));
		} catch (err) {
			console.error('Nudge failed', err);
		}
	}

	/** Stretch the region across the presentation display, sitting on the bottom margin. */
	async function snapToBottom() {
		if (!isTauri()) return;
		try {
			const { getCurrentWindow, currentMonitor, LogicalSize, LogicalPosition } =
				await import('@tauri-apps/api/window');
			const win = getCurrentWindow();
			const monitor = await currentMonitor();
			if (!monitor) return;
			// Monitor geometry is physical; window setters take logical pixels.
			const bounds = monitor.size.toLogical(monitor.scaleFactor);
			const corner = monitor.position.toLogical(monitor.scaleFactor);
			// Height is the operator's choice — snapping only settles width and position.
			const height = (await win.innerSize()).toLogical(await win.scaleFactor()).height;
			await win.setSize(new LogicalSize(Math.round(bounds.width - 96), Math.round(height)));
			await win.setPosition(
				new LogicalPosition(
					Math.round(corner.x + 48),
					Math.round(corner.y + bounds.height - height - 40)
				)
			);
		} catch (err) {
			console.error('Snap to bottom failed', err);
		}
	}

	/** Leave move mode: click-through back on, and both windows told the region is placed. */
	async function lockIntoPlace() {
		try {
			// Also re-enables no-activate on the Rust side, so raising can't steal focus.
			await api.setOverlayClickThrough(true);
		} catch (err) {
			console.error('Failed to restore overlay click-through', err);
		}
		entryGeometry = null;
		interactive = false;
		localStorage.setItem(OVERLAY_PLACED_KEY, 'true');
		void api.emitOverlayState({ interactive: false, placed: true });
	}

	/** Escape: put the window back where move mode found it and leave without placing it. */
	async function cancelMove() {
		const geo = entryGeometry;
		if (geo && isTauri()) {
			try {
				const { getCurrentWindow, PhysicalPosition, PhysicalSize } =
					await import('@tauri-apps/api/window');
				const win = getCurrentWindow();
				await win.setSize(new PhysicalSize(geo.width, geo.height));
				await win.setPosition(new PhysicalPosition(geo.x, geo.y));
			} catch (err) {
				// A failed restore must not trap the operator in move mode — carry on and exit.
				console.error('Could not restore the overlay geometry', err);
			}
		}
		try {
			await api.setOverlayClickThrough(true);
		} catch (err) {
			console.error('Failed to restore overlay click-through', err);
		}
		entryGeometry = null;
		interactive = false;
		// No `placed`: the move was abandoned, so the pre-flight check must still ask for it.
		void api.emitOverlayState({ interactive: false });
	}

	// The overlay only has a keyboard in move mode, and that is exactly when it may be
	// covering the operator window's own "Done moving" button (issue #11) — so both ways out
	// have to be reachable from here. The +/− size keys stay live in either mode.
	function onKeyDown(e: KeyboardEvent) {
		if (e.key === '+' || e.key === '=') bump(2);
		if (e.key === '-') bump(-2);
		if (!interactive) return;

		// A focused toolbar button already answers Enter itself; hijacking it would run the
		// button and the shortcut at once.
		const onButton = e.target instanceof HTMLElement && e.target.tagName === 'BUTTON';
		const step = e.shiftKey ? 10 : 1;

		switch (e.key) {
			case 'Enter':
				if (onButton) return;
				e.preventDefault();
				void lockIntoPlace();
				return;
			case 'Escape':
				e.preventDefault();
				void cancelMove();
				return;
			case 'ArrowLeft':
				e.preventDefault();
				void nudge(-step, 0);
				return;
			case 'ArrowRight':
				e.preventDefault();
				void nudge(step, 0);
				return;
			case 'ArrowUp':
				e.preventDefault();
				void nudge(0, -step);
				return;
			case 'ArrowDown':
				e.preventDefault();
				void nudge(0, step);
				return;
		}
	}
</script>

<svelte:window onkeydown={onKeyDown} />

<div
	class="stage"
	class:interactive
	data-tauri-drag-region={interactive || undefined}
	style="--fs: {fontSize}px; --measure: {captionWidth}ch"
>
	{#if interactive}
		<!-- The overlay window *is* the caption region, so the placement chrome hugs the
		     window edges rather than being drawn inside a larger screen. Everything here is
		     pointer-events:none except the toolbar, so the stage behind stays the drag region. -->
		<div class="region" aria-hidden="true">
			<span class="handle tl"></span>
			<span class="handle tr"></span>
			<span class="handle bl"></span>
			<span class="handle br"></span>
			<span class="edge top"></span>
			<span class="edge bottom"></span>
		</div>

		<!-- Dropped in a short region: there the chrome fills the window and the placeholder
		     would run under the toolbar, which reads worse than no placeholder at all. -->
		{#if winH >= 340}
			<p class="placeholder">{$t.overlay.placeholder(fontSize)}</p>
		{/if}

		<div class="chrome">
			<div class="drag-pill">
				<svg width="13" height="13" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
					<circle cx="9" cy="6" r="1.6" />
					<circle cx="15" cy="6" r="1.6" />
					<circle cx="9" cy="12" r="1.6" />
					<circle cx="15" cy="12" r="1.6" />
					<circle cx="9" cy="18" r="1.6" />
					<circle cx="15" cy="18" r="1.6" />
				</svg>
				<span class="drag-label">{$t.overlay.dragToPlace}</span>
				<span class="drag-size">{winW} × {winH}</span>
			</div>

			<div class="toolbar">
				<div class="mode">
					<span class="mode-title">{$t.overlay.moveMode}</span>
					<span class="mode-sub">{$t.overlay.paused}</span>
					<!-- The operator's own controls can be hidden under this window, so the way
					     out has to be printed where the operator is already looking. -->
					<span class="keys">
						<kbd>{$t.overlay.keyEnter}</kbd>
						{$t.overlay.keysLocks} · <kbd>{$t.overlay.keyEscape}</kbd>
						{$t.overlay.keysCancels} · <kbd>{$t.overlay.keyArrows}</kbd>
						{$t.overlay.keysNudge}
					</span>
				</div>
				<span class="divider"></span>
				<div class="size">
					<span class="size-label">{$t.overlay.size}</span>
					<button class="step" onclick={() => bump(-2)} aria-label={$t.overlay.smaller}>−</button>
					<span class="size-value">{fontSize}</span>
					<button class="step" onclick={() => bump(2)} aria-label={$t.overlay.larger}>+</button>
				</div>
				<span class="divider"></span>
				<button class="ghost" onclick={snapToBottom}>{$t.overlay.snapToBottom}</button>
				<button class="primary" onclick={lockIntoPlace}>
					<svg
						width="13"
						height="13"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						aria-hidden="true"
					>
						<rect x="4.5" y="10.5" width="15" height="10" rx="2.5" />
						<path d="M8 10.5V8a4 4 0 0 1 8 0v2.5" />
					</svg>
					{$t.overlay.lock}
				</button>
			</div>
		</div>
	{:else if lines.length > 0}
		<!-- Painted only while there is something to read: with no captions the window must
		     paint nothing at all, or it would veil the presenter's slides. -->
		<div class="captions">
			{#each lines as line (line.origin)}
				<div class="row">
					{#if showLabels}<span class="origin">{originLabel[line.origin]}</span>{/if}
					<!-- One block per speaker: dimmed lead-in, then the live text, as running text.
					     The separating space is explicit — Svelte trims literal whitespace here. -->
					<!-- prettier-ignore -->
					<p class="line" class:final={!line.interim}>
						{#if line.lead}<span class="lead">{line.lead}</span>{' '}{/if}{line.text}{#if line.interim}<span class="caret"></span>{/if}
					</p>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	/* The window itself is transparent (Tauri transparent:true). Nothing here may paint a
	   full-window background in audience view — the scrim is the only ink, and only while
	   captions are on screen. */
	:global(html),
	:global(body) {
		background: transparent !important;
	}
	.stage {
		position: fixed;
		inset: 0;
		pointer-events: none;
		user-select: none;
	}
	.stage.interactive {
		pointer-events: auto;
		cursor: move;
	}

	/* ---- Audience view ---------------------------------------------------- */

	/* The backing fade is the container's own background, so it is exactly as tall as the
	   captions plus the padded fade-out above them — a light veil the slide shows through,
	   not a band that blanks its lower half. The text shadow carries the legibility. */
	.captions {
		position: absolute;
		left: 0;
		right: 0;
		bottom: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 18px;
		padding: 9vh 6.5vw 6vh;
		pointer-events: none;
		background: linear-gradient(
			to top,
			rgba(6, 8, 10, 0.72) 0%,
			rgba(6, 8, 10, 0.42) 55%,
			rgba(6, 8, 10, 0) 100%
		);
	}
	/* Centred like cinema subtitles; with labels on, the label+text pair centres as a unit. */
	.row {
		display: flex;
		justify-content: center;
		align-items: baseline;
		gap: 20px;
	}
	.origin {
		flex: 0 0 auto;
		font-weight: 600;
		/* Scales with the caption so the label stays legible at projector distance. */
		font-size: max(11px, calc(var(--fs) * 0.37));
		line-height: 1;
		letter-spacing: 0.16em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.62);
	}
	.line {
		margin: 0;
		/* Operator-chosen, in `ch` so a measure survives a font-size change. It is a cap, not
		   a width: a region narrower than the measure simply wraps sooner, which is why no
		   setting here can push text outside the caption region. */
		max-width: var(--measure);
		font-weight: 600;
		font-size: var(--fs);
		line-height: 1.34;
		letter-spacing: -0.005em;
		text-align: center;
		color: #ffffff;
		/* A tight dark edge plus a soft halo: keeps white text readable over a bright
		   slide even where the fade above has thinned to nothing. */
		text-shadow:
			0 1px 3px rgba(0, 0, 0, 0.9),
			0 2px 14px rgba(0, 0, 0, 0.8);
		text-wrap: pretty;
	}
	/* A finished line loses a little weight of colour, never size or slant: both cost
	   legibility at the back of a room. */
	.line.final {
		color: rgba(255, 255, 255, 0.9);
	}
	/* The lead-in is the same size and sits in the same block as the live text — only its
	   contrast drops, so a sentence spanning two turns still reads as one line. */
	.lead {
		color: rgba(255, 255, 255, 0.52);
	}
	/* Marks the live turn. `blink` is a global keyframe (app.css). */
	.caret {
		display: inline-block;
		width: 4px;
		height: 0.86em;
		margin-left: 10px;
		vertical-align: -1px;
		background: #5ad1a0;
		animation: blink 1.1s steps(1) infinite;
	}

	/* ---- Move mode -------------------------------------------------------- */

	.region {
		position: absolute;
		inset: 0;
		border: 2px solid #5ad1a0;
		background: rgba(90, 209, 160, 0.07);
		pointer-events: none;
	}
	/* Affordances only: the resize itself is the OS window edge-drag. */
	.handle {
		position: absolute;
		width: 11px;
		height: 11px;
		border-radius: 3px;
		background: #5ad1a0;
	}
	.handle.tl {
		left: 3px;
		top: 3px;
	}
	.handle.tr {
		right: 3px;
		top: 3px;
	}
	.handle.bl {
		left: 3px;
		bottom: 3px;
	}
	.handle.br {
		right: 3px;
		bottom: 3px;
	}
	.edge {
		position: absolute;
		left: 50%;
		transform: translateX(-50%);
		width: 34px;
		height: 9px;
		border-radius: 3px;
		background: rgba(90, 209, 160, 0.55);
	}
	.edge.top {
		top: 3px;
	}
	.edge.bottom {
		bottom: 3px;
	}
	.placeholder {
		position: absolute;
		inset: 0;
		display: grid;
		place-items: center;
		margin: 0;
		padding: 0 34px;
		font-weight: 600;
		/* Never larger than the caption it stands in for, and never so large it wraps to
		   nothing in a short region. */
		font-size: min(34px, var(--fs));
		line-height: 1.3;
		text-align: center;
		text-wrap: pretty;
		color: rgba(255, 255, 255, 0.55);
		pointer-events: none;
	}

	/* The pill and toolbar float just inside the top edge: in the real window there is no
	   surrounding screen to hang them on. */
	.chrome {
		position: absolute;
		top: 14px;
		left: 50%;
		transform: translateX(-50%);
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		pointer-events: none;
	}
	/* Stays transparent to the pointer so dragging it drags the window (the stage below
	   carries data-tauri-drag-region). */
	.drag-pill {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 6px 12px;
		border-radius: 8px;
		background: #5ad1a0;
		color: #05271b;
		pointer-events: none;
	}
	.drag-label {
		font-weight: 600;
		font-size: var(--type-12);
		line-height: 1;
	}
	.drag-size {
		font-family: var(--font-mono);
		font-weight: 500;
		font-size: var(--type-11-5);
		line-height: 1;
		font-variant-numeric: tabular-nums;
		opacity: 0.72;
	}

	.toolbar {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 12px 14px;
		border: 1px solid #2f3540;
		border-radius: 14px;
		/* Nearly opaque, because what sits behind this window is a slide nobody controls: at
		   0.92 a white slide lifted the panel enough to cost the dimmest text its 4.5:1. */
		background: rgba(14, 17, 20, 0.96);
		box-shadow: 0 24px 60px -20px rgba(0, 0, 0, 0.8);
		color: #e9ebef;
		/* Clickable while the rest of the stage drags the window. */
		pointer-events: auto;
		cursor: default;
	}
	.mode {
		display: flex;
		flex-direction: column;
		gap: 3px;
		padding-right: 4px;
	}
	.mode-title {
		font-weight: 600;
		font-size: var(--type-10-5);
		line-height: 1;
		letter-spacing: 0.14em;
		text-transform: uppercase;
		color: #ffb454;
	}
	.mode-sub {
		font-size: var(--type-11-5);
		line-height: 1;
		color: #8b93a1;
	}
	.keys {
		margin-top: 3px;
		font-family: var(--font-mono);
		font-size: var(--type-10-5);
		line-height: 1.7;
		/* The dim end of the shared text ramp (--muted-2); spelled out because this window
		   paints over an unknown desktop and does not inherit the operator's surfaces. */
		color: #848c99;
		white-space: nowrap;
	}
	.keys kbd {
		padding: 3px 5px;
		border: 1px solid #2a2f38;
		border-radius: 5px;
		background: #191d23;
		font-family: inherit;
		font-weight: 500;
		font-size: inherit;
		color: #b9c0ca;
	}
	.divider {
		width: 1px;
		height: 30px;
		background: #2a2f38;
	}
	.size {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.size-label {
		font-size: var(--type-11-5);
		line-height: 1;
		color: #8b93a1;
	}
	.size-value {
		min-width: 24px;
		font-family: var(--font-mono);
		font-weight: 500;
		font-size: var(--type-12-5);
		line-height: 1;
		font-variant-numeric: tabular-nums;
		text-align: center;
	}
	.step {
		display: grid;
		place-items: center;
		width: 28px;
		height: 28px;
		border: 1px solid #2a2f38;
		border-radius: 7px;
		background: #171b21;
		color: #c3c9d2;
		font-weight: 500;
		font-size: var(--type-13);
		line-height: 1;
	}
	.step:hover {
		border-color: #3a4150;
		color: #e9ebef;
	}
	.ghost {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 9px 13px;
		border: 1px solid #2a2f38;
		border-radius: 9px;
		background: #171b21;
		color: #c3c9d2;
		font-weight: 500;
		font-size: var(--type-12);
		line-height: 1;
	}
	.ghost:hover {
		border-color: #3a4150;
		color: #e9ebef;
	}
	.primary {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 15px;
		border: 0;
		border-radius: 9px;
		background: linear-gradient(#5ad1a0, #43b989);
		color: #05271b;
		font-weight: 600;
		font-size: var(--type-12-5);
		line-height: 1;
	}
	.primary:hover {
		filter: brightness(1.06);
	}

	/* Windows contrast themes. The audience view opts out entirely: these captions are the
	   content being projected into a room, not application chrome, and repainting them in the
	   operator's system palette would put system-coloured text over a scrim built for white.
	   The move-mode chrome, which is chrome, keeps the system palette — it only has to drop the
	   gradient the forced palette would not have recoloured. */
	@media (forced-colors: active) {
		.captions,
		.placeholder,
		.region,
		.handle,
		.edge {
			forced-color-adjust: none;
		}
		.primary {
			background-image: none;
		}
	}
</style>
