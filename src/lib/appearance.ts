// The overlay's appearance as one value: what a meeting profile saves, what Reset restores,
// what a preset sets, and what every push to the overlay carries.
//
// The eight settings are persisted one key each (the overlay window reads them at start-up
// from the same localStorage origin), so the stores stay separate. Everything that treats
// them as a whole goes through this module, so adding a setting is one edit to the type and
// the places the compiler then points at, not a hunt through the components.

import {
	clampHex,
	clampScrimOpacity,
	DEFAULT_CAPTION_PALETTE,
	loadCaptionPalette,
	SCRIM_OPACITY_MAX,
	type CaptionPalette
} from './captionColour';
import {
	DEFAULT_CAPTION_FACE,
	isCaptionFace,
	loadCaptionFace,
	type CaptionFaceId
} from './captionFont';
import {
	DEFAULT_CAPTION_LAYOUT,
	isCaptionLayout,
	loadCaptionLayout,
	type CaptionLayout
} from './captionLayout';
import { loadCleanSpeech } from './cleanSpeech';
import {
	DEFAULT_HOLD_SECONDS,
	holdSeconds,
	loadHoldSeconds,
	loadPace,
	type CaptionPace
} from './reading';
import {
	clampOverlayFont,
	clampOverlayWidth,
	DEFAULT_OVERLAY_FONT,
	DEFAULT_OVERLAY_WIDTH,
	loadOverlayFont,
	loadOverlayWidth,
	type OverlayConfig
} from './types';

export interface Appearance {
	fontSize: number;
	/** Compact line measure, in `ch`. */
	width: number;
	layout: CaptionLayout;
	face: CaptionFaceId;
	palette: CaptionPalette;
	/** Hide filler words in the overlay only. */
	cleanSpeech: boolean;
	/** Seconds a finished Fit/Compact caption stays on screen. */
	hold: number;
	pace: CaptionPace;
}

/** What the app ships with, and what Reset appearance restores. */
export const DEFAULT_APPEARANCE: Readonly<Appearance> = Object.freeze({
	fontSize: DEFAULT_OVERLAY_FONT,
	width: DEFAULT_OVERLAY_WIDTH,
	layout: DEFAULT_CAPTION_LAYOUT,
	face: DEFAULT_CAPTION_FACE,
	palette: Object.freeze({ ...DEFAULT_CAPTION_PALETTE }),
	cleanSpeech: false,
	hold: DEFAULT_HOLD_SECONDS,
	pace: 'immediate'
});

/** A complete, valid appearance from anything: a stored profile, a patch, or garbage. Each
 *  setting falls back to its own default, so one bad field does not discard the rest. */
export function normalizeAppearance(value: unknown): Appearance {
	const a = (value && typeof value === 'object' ? value : {}) as Partial<Appearance>;
	const finite = (n: unknown): n is number => typeof n === 'number' && Number.isFinite(n);
	return {
		fontSize: finite(a.fontSize) ? clampOverlayFont(a.fontSize) : DEFAULT_APPEARANCE.fontSize,
		width: finite(a.width) ? clampOverlayWidth(a.width) : DEFAULT_APPEARANCE.width,
		layout: isCaptionLayout(a.layout) ? a.layout : DEFAULT_APPEARANCE.layout,
		face: isCaptionFace(a.face) ? a.face : DEFAULT_APPEARANCE.face,
		palette: {
			text: clampHex(a.palette?.text, DEFAULT_CAPTION_PALETTE.text),
			scrim: clampHex(a.palette?.scrim, DEFAULT_CAPTION_PALETTE.scrim),
			scrimOpacity: clampScrimOpacity(
				a.palette?.scrimOpacity ?? DEFAULT_CAPTION_PALETTE.scrimOpacity
			)
		},
		cleanSpeech: a.cleanSpeech === true,
		hold: holdSeconds(a.hold),
		pace: a.pace === 'steady' ? 'steady' : 'immediate'
	};
}

/** The appearance persisted by the operator window, as the overlay reads it at start-up. */
export function loadAppearance(): Appearance {
	return {
		fontSize: loadOverlayFont(),
		width: loadOverlayWidth(),
		layout: loadCaptionLayout(),
		face: loadCaptionFace(),
		palette: loadCaptionPalette(),
		cleanSpeech: loadCleanSpeech(),
		hold: loadHoldSeconds(),
		pace: loadPace()
	};
}

export function sameAppearance(a: Readonly<Appearance>, b: Readonly<Appearance>): boolean {
	return (
		a.fontSize === b.fontSize &&
		a.width === b.width &&
		a.layout === b.layout &&
		a.face === b.face &&
		samePalette(a.palette, b.palette) &&
		a.cleanSpeech === b.cleanSpeech &&
		a.hold === b.hold &&
		a.pace === b.pace
	);
}

function samePalette(a: CaptionPalette, b: CaptionPalette): boolean {
	return a.text === b.text && a.scrim === b.scrim && a.scrimOpacity === b.scrimOpacity;
}

/** The appearance fields of an overlay push. The event keeps its own names, because a
 *  reloaded overlay from an older build still has to understand it. */
export function toOverlayConfig(a: Readonly<Appearance>): OverlayConfig {
	return {
		fontSize: a.fontSize,
		captionWidth: a.width,
		captionLayout: a.layout,
		captionFace: a.face,
		captionColour: a.palette.text,
		scrimColour: a.palette.scrim,
		scrimOpacity: a.palette.scrimOpacity,
		cleanSpeech: a.cleanSpeech,
		holdSeconds: a.hold,
		pace: a.pace
	};
}

export type PresetId = 'standard' | 'projector' | 'contrast';
export const PRESET_IDS: readonly PresetId[] = ['standard', 'projector', 'contrast'];

/** Reading presets. Each sets size, face, layout and colours; timing and filler cleanup are
 *  left as the operator had them. Values must already be in range: a preset that normalized
 *  to something else could never show as the current one. High contrast once asked for an
 *  opaque scrim, was stored at the 0.95 cap, and so never read as pressed. */
export const PRESETS: Readonly<
	Record<PresetId, Readonly<Pick<Appearance, 'fontSize' | 'face' | 'layout' | 'palette'>>>
> = {
	standard: {
		fontSize: DEFAULT_OVERLAY_FONT,
		face: DEFAULT_CAPTION_FACE,
		layout: 'fit',
		palette: { ...DEFAULT_CAPTION_PALETTE }
	},
	projector: {
		fontSize: 52,
		face: DEFAULT_CAPTION_FACE,
		layout: 'stable',
		palette: { text: '#ffffff', scrim: '#000000', scrimOpacity: 0.85 }
	},
	contrast: {
		fontSize: DEFAULT_OVERLAY_FONT,
		face: DEFAULT_CAPTION_FACE,
		layout: 'fit',
		palette: { text: '#ffffff', scrim: '#000000', scrimOpacity: SCRIM_OPACITY_MAX }
	}
};

export function matchesPreset(a: Readonly<Appearance>, id: PresetId): boolean {
	const preset = PRESETS[id];
	return (
		a.fontSize === preset.fontSize &&
		a.face === preset.face &&
		a.layout === preset.layout &&
		samePalette(a.palette, preset.palette)
	);
}
