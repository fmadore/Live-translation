import { get } from 'svelte/store';
import { api } from './tauri';
import { asStatus } from './errors';
import {
	options,
	overlayFontSize,
	overlayCaptionWidth,
	overlayCaptionFace,
	overlayPalette,
	overlayPlaced,
	statusMessage
} from './stores';
import {
	captionLanguageOf,
	clampOverlayFont,
	clampOverlayWidth,
	DEFAULT_OVERLAY_FONT,
	DEFAULT_OVERLAY_WIDTH
} from './types';
import type { OverlayConfig, OverlayStateMsg } from './types';
import { clampHex, clampScrimOpacity, DEFAULT_CAPTION_PALETTE } from './captionColour';
import type { CaptionPalette } from './captionColour';
import {
	availableCaptionFaces,
	CAPTION_FACES,
	DEFAULT_CAPTION_FACE,
	measureWithCanvas
} from './captionFont';
import type { CaptionFace, CaptionFaceId } from './captionFont';

/** One appearance and window-command owner for the rail, settings and overlay events. */
export function createOverlayController(port = api) {
	const api = port;
	/** The faces this machine can actually render. Starts as the whole list and narrows on
	 *  mount, once there is a canvas to measure with — offering a face and finding out later
	 *  that it silently fell back is the failure this avoids. */
	let captionFaces = $state<readonly CaptionFace[]>(CAPTION_FACES);

	/** Read the current audience language when building a complete configuration. */
	const captionLanguage = () => captionLanguageOf(get(options));

	// Every push carries the whole appearance, not the field that changed: the overlay is a
	// separate webview that can be reloaded independently, and a partial config would leave
	// it showing whatever it had before. One helper so no call site can forget a field.
	function pushOverlayConfig(extra: Partial<OverlayConfig> = {}) {
		void api
			.setOverlayConfig({
				fontSize: get(overlayFontSize),
				captionWidth: get(overlayCaptionWidth),
				captionFace: get(overlayCaptionFace),
				captionColour: get(overlayPalette).text,
				scrimColour: get(overlayPalette).scrim,
				scrimOpacity: get(overlayPalette).scrimOpacity,
				captionLanguage: captionLanguage(),
				...extra
			})
			.catch((e) => statusMessage.set(asStatus(e)));
	}

	// Caption size: update the store (persists) and push it live to the overlay.
	function setFont(size: number) {
		overlayFontSize.set(clampOverlayFont(size));
		pushOverlayConfig({ interactive: moveOverlay });
	}

	// Caption measure: how long a line is allowed to run before it wraps. Same shape as the
	// size control, and the same live push.
	function setCaptionWidth(width: number) {
		overlayCaptionWidth.set(clampOverlayWidth(width));
		pushOverlayConfig({ interactive: moveOverlay });
	}

	/** Change part of the palette. Clamped here as well as on the way in to the overlay: the
	 *  operator window is where the contrast readout is computed, and a readout describing a
	 *  colour the overlay would refuse to paint would be worse than no readout. */
	function setPalette(patch: Partial<CaptionPalette>) {
		overlayPalette.update((current) => {
			const next = { ...current, ...patch };
			return {
				text: clampHex(next.text, DEFAULT_CAPTION_PALETTE.text),
				scrim: clampHex(next.scrim, DEFAULT_CAPTION_PALETTE.scrim),
				scrimOpacity: clampScrimOpacity(next.scrimOpacity)
			};
		});
		pushOverlayConfig({ interactive: moveOverlay });
	}

	/** Put the overlay's whole appearance back to what it ships with.
	 *
	 *  Everything in this section, not just the colours: a palette that has gone wrong has
	 *  usually gone wrong alongside a size and a measure that were moved trying to fix it, and
	 *  a reset that left those behind would not be the way out it is reached for. Placement is
	 *  deliberately untouched — that is where the window sits on the projector, it took a walk
	 *  across the room to get right, and nothing here is a reason to lose it. */
	function resetOverlayAppearance() {
		overlayFontSize.set(DEFAULT_OVERLAY_FONT);
		overlayCaptionWidth.set(DEFAULT_OVERLAY_WIDTH);
		overlayCaptionFace.set(DEFAULT_CAPTION_FACE);
		overlayPalette.set({ ...DEFAULT_CAPTION_PALETTE });
		pushOverlayConfig({ interactive: moveOverlay });
	}

	function setCaptionFace(id: CaptionFaceId) {
		overlayCaptionFace.set(id);
		// Carrying move mode through, like the size and the measure: the operator is usually
		// looking at the overlay while choosing, and a push that dropped it would snap the
		// window back to click-through mid-adjustment.
		pushOverlayConfig({ interactive: moveOverlay });
	}

	// Move mode: the overlay is click-through while captioning; this flips it into an
	// interactive drag region so it can be dragged/resized into place, then flipped back.
	// The overlay can also leave move mode on its own (its Enter/Escape keys), which arrives
	// as an overlayState event — so this flag is the single source of truth, never cached.
	let moveOverlay = $state(false);

	// The overlay window is created visible (tauri.conf.json), so the toggle starts on "Hide".
	// Blanking it covers a coffee break or a video clip without ending the session.
	let overlayVisible = $state(true);

	async function toggleMoveOverlay() {
		const next = !moveOverlay;
		try {
			await api.showOverlay(true);
			overlayVisible = true;
			await api.setOverlayClickThrough(!next);
			moveOverlay = next;
			pushOverlayConfig({ interactive: moveOverlay });
		} catch (e) {
			statusMessage.set(asStatus(e));
		}
	}

	async function toggleOverlayVisible() {
		const next = !overlayVisible;
		try {
			await api.showOverlay(next);
			overlayVisible = next;
		} catch (e) {
			statusMessage.set(asStatus(e));
		}
	}

	function initialize() {
		// Which faces this machine has. Measured here rather than at module load so the
		// bundled webfont has had a chance to arrive first — see `availableCaptionFaces`.
		captionFaces = availableCaptionFaces(measureWithCanvas());
		// The choice persists, the font does not: a face uninstalled since it was chosen would
		// leave the control showing one thing and the overlay painting its fallback. Settle it
		// back to the bundled default instead of letting the two disagree.
		if (!captionFaces.some((f) => f.id === get(overlayCaptionFace)))
			setCaptionFace(DEFAULT_CAPTION_FACE);
	}

	function applyState(msg: OverlayStateMsg) {
		if (msg.interactive === false) moveOverlay = false;
		if (msg.placed === true) overlayPlaced.set(true);
		if (typeof msg.fontSize === 'number' && Number.isFinite(msg.fontSize))
			overlayFontSize.set(clampOverlayFont(msg.fontSize));
	}

	return {
		get captionFaces() {
			return captionFaces;
		},
		get moveOverlay() {
			return moveOverlay;
		},
		get overlayVisible() {
			return overlayVisible;
		},
		initialize,
		applyState,
		pushOverlayConfig,
		setFont,
		setCaptionWidth,
		setPalette,
		resetOverlayAppearance,
		setCaptionFace,
		toggleMoveOverlay,
		toggleOverlayVisible
	};
}
export type OverlayController = ReturnType<typeof createOverlayController>;
