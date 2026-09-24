<script lang="ts">
	import { onMount } from 'svelte';
	import OverlayCaptionLine from './OverlayCaptionLine.svelte';
	import OverlayMoveChrome from './OverlayMoveChrome.svelte';
	import { createOverlayCaptions } from './overlayCaptions.svelte';
	import { createOverlayPlacement, overlayKeyCommand } from './overlayPlacement.svelte';
	import { previewContent } from './overlayFixtures';
	import { loadAppearance } from '$lib/appearance';
	import { loadFillerWords, normalizeFillerWords } from '$lib/cleanSpeech';
	import { bottomCaptionHeight, isCaptionLayout } from '$lib/captionLayout';
	import {
		captionCssVars,
		clampHex,
		clampScrimOpacity,
		DEFAULT_CAPTION_PALETTE,
		type CaptionPalette
	} from '$lib/captionColour';
	import { captionFaceStack, isCaptionFace, type CaptionFaceId } from '$lib/captionFont';
	import { isLocale, locale, t } from '$lib/i18n';
	import { isTargetLanguage } from '$lib/languages';
	import { api, on, isTauri } from '$lib/tauri';
	import { clampOverlayFont, type Origin, type TargetLanguage } from '$lib/types';

	// The initial appearance comes from the shared localStorage keys (same origin as the
	// operator), then the operator pushes live updates via the overlay-config event.
	const initial = loadAppearance();
	const captions = createOverlayCaptions({
		layout: initial.layout,
		hideFillers: initial.cleanSpeech,
		fillerWords: loadFillerWords(),
		hold: initial.hold,
		pace: initial.pace,
		width: initial.width
	});
	const placement = createOverlayPlacement();

	let fontSize = $state(initial.fontSize);
	let fontsLoaded = $state(0);
	let captionFace = $state<CaptionFaceId>(initial.face);

	// The ink and the scrim behind it. The overlay keeps the operator's three plain values and
	// derives its own steps from them, rather than being handed a finished stylesheet — so the
	// contrast readout on the control panel and the pixels here come out of one function.
	let palette = $state<CaptionPalette>(initial.palette);
	const paletteVars = $derived(
		Object.entries(captionCssVars(palette))
			.map(([name, value]) => `${name}: ${value}`)
			.join('; ')
	);

	// The language of the caption text, which is not this window's interface language: the
	// move-mode chrome and the origin labels follow the operator's locale, the captions do
	// not. Only the operator window can work it out, so it is pushed. Undefined means unknown
	// — before the first push, and while a subtitle engine is detecting the spoken language.
	let captionLanguage = $state<TargetLanguage | undefined>(undefined);

	// Words, not emoji or colour: at projector distance a two-letter cue is unreadable.
	const originLabel = $derived<Record<Origin, string>>($t.overlay.origin);

	// Live window size, for the move-mode readout and the row heights. The overlay window *is*
	// the caption region, so its own viewport is the number that matters.
	let winW = $state(0);
	let winH = $state(0);

	const lines = $derived(captions.lines);
	const layout = $derived(captions.layout);
	// A single speaker needs no label — the row is unambiguous, and the label would only steal
	// width from the caption. Labels appear exactly when both origins are on screen.
	const showLabels = $derived(lines.length > 1);
	const sidePadding = $derived(
		layout !== 'compact' ? Math.min(32, Math.max(12, winW * 0.025)) : winW * 0.065
	);
	const topPadding = $derived(layout !== 'compact' ? 16 : winH * 0.09);
	const bottomPadding = $derived(layout !== 'compact' ? 16 : winH * 0.06);
	const rowHeight = $derived(
		Math.max(
			0,
			(winH - topPadding - bottomPadding - Math.max(0, lines.length - 1) * 18) /
				Math.max(1, lines.length)
		)
	);

	onMount(() => {
		const measure = () => {
			winW = Math.round(window.innerWidth);
			winH = Math.round(window.innerHeight);
		};
		measure();
		let mounted = true;
		void document.fonts.ready.then(() => {
			if (mounted) fontsLoaded += 1;
		});
		const fontsChanged = () => {
			fontsLoaded += 1;
		};
		document.fonts.addEventListener('loadingdone', fontsChanged);
		window.addEventListener('resize', measure);
		const cleanup = () => {
			mounted = false;
			window.removeEventListener('resize', measure);
			document.fonts.removeEventListener('loadingdone', fontsChanged);
			captions.dispose();
		};

		if (!isTauri()) {
			// A browser preview has no core: show labelled demo content instead.
			const preview = previewContent(new URLSearchParams(window.location.search).get('language'));
			captions.show(preview);
			if (preview.language) captionLanguage = preview.language;
			return cleanup;
		}

		const unlistenCaption = on.caption((c) => captions.push(c));
		const unlistenStatus = on.status((status) => captions.status(status));
		const unlistenConfig = on.overlayConfig((cfg) => {
			if (typeof cfg.holdSeconds === 'number') captions.setHold(cfg.holdSeconds);
			if (cfg.pace === 'steady' || cfg.pace === 'immediate') captions.setPace(cfg.pace);
			if (typeof cfg.cleanSpeech === 'boolean') captions.setHideFillers(cfg.cleanSpeech);
			if (Array.isArray(cfg.fillerWords))
				captions.setFillerWords(normalizeFillerWords(cfg.fillerWords));
			if (isCaptionLayout(cfg.captionLayout)) captions.setLayout(cfg.captionLayout);
			if (Number.isFinite(cfg.fontSize) && cfg.fontSize > 0)
				fontSize = clampOverlayFont(cfg.fontSize);
			if (Number.isFinite(cfg.captionWidth) && (cfg.captionWidth ?? 0) > 0)
				captions.setWidth(cfg.captionWidth as number);
			// An id, not a stack: what arrives over the event is checked against the faces this
			// build knows, so nothing here can put an arbitrary `font-family` on the screen an
			// audience is reading.
			if (isCaptionFace(cfg.captionFace)) captionFace = cfg.captionFace;
			// Validated here as well as at the source. Every value is clamped to something
			// paintable, each independently, so one bad field cannot take the palette down —
			// a caption in an unparsed colour is a caption in no colour at all.
			palette = {
				text: clampHex(cfg.captionColour ?? palette.text, DEFAULT_CAPTION_PALETTE.text),
				scrim: clampHex(cfg.scrimColour ?? palette.scrim, DEFAULT_CAPTION_PALETTE.scrim),
				scrimOpacity: clampScrimOpacity(cfg.scrimOpacity ?? palette.scrimOpacity)
			};
			// The operator owns the interface language; this window follows it.
			if (isLocale(cfg.locale)) locale.set(cfg.locale);
			// Unconditional, unlike the rest: an absent caption language is a real answer
			// ("nobody knows"), so it has to be able to clear one that was set before.
			captionLanguage = isTargetLanguage(cfg.captionLanguage) ? cfg.captionLanguage : undefined;
			if (typeof cfg.interactive === 'boolean') placement.setInteractive(cfg.interactive);
		});

		return () => {
			cleanup();
			void unlistenCaption.then((f) => f());
			void unlistenStatus.then((f) => f());
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

	/** Reset an enlarged reading region to a shallow subtitle strip along the bottom. */
	function snapToBottom() {
		void placement.snapToBottom(bottomCaptionHeight(fontSize, lines.length, layout));
	}

	function onKeyDown(e: KeyboardEvent) {
		const command = overlayKeyCommand(
			{
				key: e.key,
				shiftKey: e.shiftKey,
				onButton: e.target instanceof HTMLElement && e.target.tagName === 'BUTTON'
			},
			placement.interactive
		);
		if (!command) return;
		if (command.kind === 'bump') {
			bump(command.delta);
			return;
		}
		e.preventDefault();
		if (command.kind === 'lock') void placement.lock();
		else if (command.kind === 'cancel') void placement.cancel();
		else void placement.nudge(command.dx, command.dy);
	}
</script>

<svelte:window onkeydown={onKeyDown} />

<div
	class="stage"
	class:interactive={placement.interactive}
	class:stable={layout === 'stable'}
	class:fit={layout !== 'compact'}
	data-tauri-drag-region={placement.interactive || undefined}
	style="--side-pad: {sidePadding}px; --top-pad: {topPadding}px; --bottom-pad: {bottomPadding}px; --fs: {fontSize}px; --measure: {captions.width}ch; --caption-face: {captionFaceStack(
		captionFace
	)}; {paletteVars}"
>
	{#if placement.interactive}
		<OverlayMoveChrome
			{fontSize}
			width={winW}
			height={winH}
			onBump={bump}
			onSnap={snapToBottom}
			onLock={() => void placement.lock()}
		/>
	{:else if lines.length > 0}
		<!-- Painted only while there is something to read: with no captions the window must
		     paint nothing at all, or it would veil the presenter's slides. -->
		<div class="captions">
			{#each lines as line (line.origin)}
				<div class="row">
					<!-- The label is interface-language text sitting beside caption-language text, and
					     it inherits `<html lang>`, which is the interface language. Correct as it is. -->
					{#if showLabels}<span class="origin">{originLabel[line.origin]}</span>{/if}
					<OverlayCaptionLine
						stable={layout === 'stable'}
						onTrim={(chars) => captions.trimStable(line.origin, chars)}
						lead={line.lead}
						text={line.text}
						interim={line.interim}
						height={rowHeight}
						fontKey={fontSize + ':' + captionFace + ':' + fontsLoaded}
						language={captionLanguage ?? ''}
					/>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.stable .captions {
		top: 0;
		bottom: auto;
		background: var(--caption-scrim-strong);
	}
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
		padding: var(--top-pad) var(--side-pad) var(--bottom-pad);
		max-height: 100%;
		overflow: hidden;
		pointer-events: none;
		/* The audience view only. Move-mode chrome stays on the app's own face: it is this
		   operator's interface, not the thing being projected. */
		font-family: var(--caption-face);
		/* Operator-chosen, and the shape is the point: strongest at the bottom edge, thinner
		   where the text sits, gone at the top. Moving the opacity scales all three stops
		   together, so the veil never becomes a band with an edge. */
		background: linear-gradient(
			to top,
			var(--caption-scrim-strong) 0%,
			var(--caption-scrim-mid) 55%,
			var(--caption-scrim-none) 100%
		);
	}
	/* Centred like cinema subtitles; with labels on, the label+text pair centres as a unit. */
	.row {
		width: 100%;
		min-height: 0;
		max-width: var(--measure);
		font-size: var(--fs);
		display: flex;
		justify-content: center;
		align-items: baseline;
		gap: 20px;
	}
	.fit .row {
		max-width: none;
	}
	.origin {
		flex: 0 0 auto;
		font-weight: 600;
		/* Scales with the caption so the label stays legible at projector distance. */
		font-size: max(11px, calc(var(--fs) * 0.37));
		line-height: 1;
		letter-spacing: 0.16em;
		text-transform: uppercase;
		color: var(--caption-ink-label);
	}

	/* Windows contrast themes. The audience view opts out entirely: these captions are the
	   content being projected into a room, not application chrome, and repainting them in the
	   operator's system palette would put system-coloured text over a scrim built for white.
	   The move-mode chrome keeps its own rules; see OverlayMoveChrome. */
	@media (forced-colors: active) {
		.captions {
			forced-color-adjust: none;
		}
	}
</style>
