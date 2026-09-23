import { describe, expect, it } from 'vitest';
import {
	DEFAULT_APPEARANCE,
	loadAppearance,
	matchesPreset,
	normalizeAppearance,
	PRESET_IDS,
	PRESETS,
	sameAppearance,
	toOverlayConfig
} from './appearance';

describe('the appearance schema', () => {
	it('falls back to the shipped appearance for anything unusable', () => {
		for (const value of [undefined, null, 'x', 42, {}]) {
			expect(sameAppearance(normalizeAppearance(value), DEFAULT_APPEARANCE)).toBe(true);
		}
	});

	it('keeps each valid setting and replaces only the broken ones', () => {
		const a = normalizeAppearance({
			fontSize: 60,
			width: 'wide',
			layout: 'stable',
			face: 'comic-sans',
			palette: { text: '#ffe066', scrim: 'black', scrimOpacity: 4 },
			cleanSpeech: true,
			hold: 999,
			pace: 'steady'
		});
		expect(a.fontSize).toBe(60);
		expect(a.width).toBe(DEFAULT_APPEARANCE.width);
		expect(a.layout).toBe('stable');
		expect(a.face).toBe(DEFAULT_APPEARANCE.face);
		expect(a.palette.text).toBe('#ffe066');
		expect(a.palette.scrim).toBe(DEFAULT_APPEARANCE.palette.scrim);
		expect(a.palette.scrimOpacity).toBeLessThanOrEqual(1);
		expect(a.cleanSpeech).toBe(true);
		expect(a.hold).toBe(30);
		expect(a.pace).toBe('steady');
	});

	it('returns fresh objects, so a saved profile never shares the live palette', () => {
		const a = normalizeAppearance(DEFAULT_APPEARANCE);
		expect(a.palette).not.toBe(DEFAULT_APPEARANCE.palette);
		a.palette.text = '#000000';
		expect(DEFAULT_APPEARANCE.palette.text).not.toBe('#000000');
	});

	it('loads the shipped appearance when nothing is stored', () => {
		expect(sameAppearance(loadAppearance(), DEFAULT_APPEARANCE)).toBe(true);
	});

	it('sends every setting to the overlay', () => {
		const config = toOverlayConfig(DEFAULT_APPEARANCE);
		// Eight settings, with the palette sent as its three parts.
		expect(Object.keys(config)).toHaveLength(Object.keys(DEFAULT_APPEARANCE).length - 1 + 3);
		expect(Object.values(config)).not.toContain(undefined);
		expect(config).toMatchObject({
			fontSize: DEFAULT_APPEARANCE.fontSize,
			captionWidth: DEFAULT_APPEARANCE.width,
			holdSeconds: DEFAULT_APPEARANCE.hold,
			scrimOpacity: DEFAULT_APPEARANCE.palette.scrimOpacity
		});
	});

	it('compares every setting, including each part of the palette', () => {
		const base = normalizeAppearance(DEFAULT_APPEARANCE);
		expect(sameAppearance(base, DEFAULT_APPEARANCE)).toBe(true);
		expect(sameAppearance({ ...base, hold: 5 }, DEFAULT_APPEARANCE)).toBe(false);
		expect(
			sameAppearance(
				{ ...base, palette: { ...base.palette, scrimOpacity: 0.5 } },
				DEFAULT_APPEARANCE
			)
		).toBe(false);
	});
});

describe('reading presets', () => {
	it.each(PRESET_IDS)('%s holds values the overlay accepts unchanged', (id) => {
		const applied = normalizeAppearance({ ...DEFAULT_APPEARANCE, ...PRESETS[id] });
		expect(matchesPreset(applied, id)).toBe(true);
	});

	it('marks exactly one preset as current after applying it', () => {
		for (const id of PRESET_IDS) {
			const applied = normalizeAppearance({ ...DEFAULT_APPEARANCE, ...PRESETS[id] });
			expect(PRESET_IDS.filter((other) => matchesPreset(applied, other))).toEqual([id]);
		}
	});

	it('shows a fresh install as Standard', () => {
		expect(matchesPreset(DEFAULT_APPEARANCE, 'standard')).toBe(true);
	});

	it('leaves reading pace, hold and filler cleanup alone', () => {
		const tuned = { ...DEFAULT_APPEARANCE, hold: 12, pace: 'steady' as const, cleanSpeech: true };
		const applied = normalizeAppearance({ ...tuned, ...PRESETS.projector });
		expect(applied).toMatchObject({ hold: 12, pace: 'steady', cleanSpeech: true });
	});
});
