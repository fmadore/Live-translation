import {
	normalizeStartOptions,
	clampOverlayFont,
	clampOverlayWidth,
	type StartOptions
} from './types';
import { isCaptionLayout, type CaptionLayout } from './captionLayout';
import { isCaptionFace, type CaptionFaceId } from './captionFont';
import {
	clampHex,
	clampScrimOpacity,
	DEFAULT_CAPTION_PALETTE,
	type CaptionPalette
} from './captionColour';
import { holdSeconds, type CaptionPace } from './reading';
export const PROFILES_KEY = 'meeting.profiles';
export interface Placement {
	x: number;
	y: number;
	width: number;
	height: number;
}
export interface Appearance {
	fontSize: number;
	width: number;
	layout: CaptionLayout;
	face: CaptionFaceId;
	palette: CaptionPalette;
	cleanSpeech: boolean;
	hold: number;
	pace: CaptionPace;
}
export interface MeetingProfile {
	id: string;
	name: string;
	options: StartOptions;
	appearance: Appearance;
	placement: Placement | null;
}
export function normalizeAppearance(value: unknown): Appearance {
	const a = (value && typeof value === 'object' ? value : {}) as Partial<Appearance>;
	return {
		fontSize:
			typeof a.fontSize === 'number' && Number.isFinite(a.fontSize)
				? clampOverlayFont(a.fontSize)
				: 38,
		width:
			typeof a.width === 'number' && Number.isFinite(a.width) ? clampOverlayWidth(a.width) : 30,
		layout: isCaptionLayout(a.layout) ? a.layout : 'fit',
		face: isCaptionFace(a.face) ? a.face : 'archivo',
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
export function decodeProfiles(raw: string | null): MeetingProfile[] {
	try {
		const data: unknown = JSON.parse(raw ?? '[]');
		if (!Array.isArray(data)) return [];
		const ids = new Set<string>();
		return data.flatMap((p) => {
			if (
				!p ||
				typeof p.id !== 'string' ||
				ids.has(p.id) ||
				typeof p.name !== 'string' ||
				!p.name.trim()
			)
				return [];
			ids.add(p.id);
			const g = p.placement;
			const valid =
				g &&
				['x', 'y', 'width', 'height'].every((k) => Number.isSafeInteger(g[k])) &&
				Math.abs(g.x) <= 2147483647 &&
				Math.abs(g.y) <= 2147483647 &&
				g.width > 0 &&
				g.height > 0 &&
				g.width <= 16384 &&
				g.height <= 16384;
			return [
				{
					id: p.id,
					name: p.name.trim().slice(0, 80),
					options: normalizeStartOptions(p.options),
					appearance: normalizeAppearance(p.appearance),
					placement: valid ? { x: g.x, y: g.y, width: g.width, height: g.height } : null
				}
			];
		});
	} catch {
		return [];
	}
}
