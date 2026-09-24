import type { CaptionPace } from './reading';
import type { CaptionLayout } from './captionLayout';
import { get } from 'svelte/store';
import { api } from './tauri';
import { asStatus } from './errors';
import {
	appearance,
	applyAppearance,
	options,
	overlayFontSize,
	overlayCaptionFace,
	overlayFillerWords,
	overlayPalette,
	overlayPlaced,
	statusMessage
} from './stores';
import { captionLanguageOf, clampOverlayFont } from './types';
import type { OverlayConfig, OverlayStateMsg } from './types';
import { DEFAULT_APPEARANCE, toOverlayConfig, type Appearance } from './appearance';
import type { CaptionPalette } from './captionColour';
import { normalizeFillerWords } from './cleanSpeech';
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
	// it showing whatever it had before. One helper so no call site can forget a field. The
	// filler word list rides along for the same reason.
	function pushOverlayConfig(extra: Partial<OverlayConfig> = {}) {
		void api
			.setOverlayConfig({
				...toOverlayConfig(get(appearance)),
				captionLanguage: captionLanguage(),
				fillerWords: [...get(overlayFillerWords)],
				...extra
			})
			.catch((e) => statusMessage.set(asStatus(e)));
	}

	/** Change part of the appearance: update the stores (which persist) and push the whole of
	 *  it live to the overlay. Normalized here as well as on the way in to the overlay: the
	 *  operator window is where the contrast readout is computed, and a readout describing a
	 *  colour the overlay would refuse to paint would be worse than no readout.
	 *
	 *  Move mode is carried through: the operator is usually looking at the overlay while
	 *  choosing, and a push that dropped it would snap the window back to click-through
	 *  mid-adjustment. */
	function setAppearance(patch: Partial<Appearance>) {
		applyAppearance({ ...get(appearance), ...patch });
		pushOverlayConfig({ interactive: moveOverlay });
	}

	const setFont = (fontSize: number) => setAppearance({ fontSize });
	const setCaptionLayout = (layout: CaptionLayout) => setAppearance({ layout });
	/** Caption measure: how long a Compact line is allowed to run before it wraps. */
	const setCaptionWidth = (width: number) => setAppearance({ width });
	const setCaptionFace = (face: CaptionFaceId) => setAppearance({ face });
	const setPalette = (patch: Partial<CaptionPalette>) =>
		setAppearance({ palette: { ...get(overlayPalette), ...patch } });

	/** Replace the words Hide filler words removes, and show the change live. Apart from
	 *  `setAppearance` because the list is apart from the appearance: see `overlayFillerWords`. */
	function setFillerWords(words: readonly string[]) {
		overlayFillerWords.set(normalizeFillerWords(words));
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
		setAppearance(DEFAULT_APPEARANCE);
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
		setAppearance,
		setHoldSeconds: (hold: number) => setAppearance({ hold }),
		setPace: (pace: CaptionPace) => setAppearance({ pace }),
		setFont,
		setCaptionWidth,
		setCaptionLayout,
		setCleanSpeech: (cleanSpeech: boolean) => setAppearance({ cleanSpeech }),
		setFillerWords,
		setPalette,
		resetOverlayAppearance,
		setCaptionFace,
		toggleMoveOverlay,
		toggleOverlayVisible
	};
}
export type OverlayController = ReturnType<typeof createOverlayController>;
