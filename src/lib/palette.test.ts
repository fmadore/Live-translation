import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

// Issue #24. The contrast failures this guards against were not written deliberately — they
// arrived one shade at a time, because nothing in the repo could tell that the dimmest grey
// had drifted under 4.5:1 on the panel it is read on. This is that check: it reads the real
// tokens out of `app.css`, so dimming one fails here rather than on a reviewer's machine.
//
// It covers the tokens and the tinted washes they name. Text on a wash the stylesheet does
// not name (a gradient, an opacity) is verified against the rendered window instead — see
// `docs/accessibility.md`.

const CSS = readFileSync(new URL('../app.css', import.meta.url), 'utf8');
const ROOT = CSS.slice(CSS.indexOf(':root'), CSS.indexOf('\n}'));

/** Every `--name: #rrggbb` in `:root`, with `--name: var(--other)` aliases resolved. Non-hex
 *  values (rgba washes) are skipped. */
function palette(): Record<string, string> {
	const out: Record<string, string> = {};
	for (const [, name, hex] of ROOT.matchAll(/(--[\w-]+):\s*(#[0-9a-fA-F]{6})\s*;/g)) {
		out[name] = hex;
	}
	for (const [, name, target] of ROOT.matchAll(/(--[\w-]+):\s*var\((--[\w-]+)\)\s*;/g)) {
		if (out[target]) out[name] = out[target];
	}
	return out;
}

/** Every `--name: rgba(r, g, b, a)` in `:root`: the tinted washes chips and cards sit on. */
function washes(): Record<string, { rgb: number[]; alpha: number }> {
	const out: Record<string, { rgb: number[]; alpha: number }> = {};
	for (const [, name, r, g, b, a] of ROOT.matchAll(
		/(--[\w-]+-bg):\s*rgba\((\d+),\s*(\d+),\s*(\d+),\s*([\d.]+)\)\s*;/g
	)) {
		out[name] = { rgb: [r, g, b].map(Number), alpha: Number(a) };
	}
	return out;
}

function rgb(hex: string): number[] {
	return [0, 1, 2].map((i) => parseInt(hex.slice(1 + i * 2, 3 + i * 2), 16));
}

function hexOf(channels: number[]): string {
	return `#${channels.map((c) => Math.round(c).toString(16).padStart(2, '0')).join('')}`;
}

/** A wash laid over a surface, as the window composites it. */
function over(wash: { rgb: number[]; alpha: number }, surface: string): string {
	const ground = rgb(surface);
	return hexOf(wash.rgb.map((c, i) => c * wash.alpha + ground[i] * (1 - wash.alpha)));
}

function luminance(hex: string): number {
	const channel = (i: number) => {
		const c = parseInt(hex.slice(1 + i * 2, 3 + i * 2), 16) / 255;
		return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
	};
	return 0.2126 * channel(0) + 0.7152 * channel(1) + 0.0722 * channel(2);
}

function contrast(a: string, b: string): number {
	const [x, y] = [luminance(a), luminance(b)];
	return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
}

/** Everything a window paints text onto, deepest first. */
const SURFACES = ['--surface-0', '--surface-1', '--surface-2'];

/** The text ramp, brightest first. */
const TEXT_LEVELS = ['--text-bright', '--text-body', '--text-secondary', '--text-muted'];

/** Everything that carries text. WCAG AA for normal-sized text is 4.5:1. */
const TEXT = [
	...TEXT_LEVELS,
	'--accent',
	'--accent-soft',
	'--room-soft',
	'--warn',
	'--warn-soft',
	'--danger',
	'--danger-soft'
];

/** Not text: a status dot, a ring, a fill. WCAG AA for a meaningful non-text mark is 3:1. */
const MARKS = ['--faint', '--focus', '--room'];

describe('the palette', () => {
	const tokens = palette();

	it('defines every token these checks reason about', () => {
		for (const name of [...SURFACES, ...TEXT, ...MARKS]) {
			expect(tokens[name], `${name} is missing from app.css`).toBeDefined();
		}
	});

	// The palette grew by accretion once: seven text greys, two groups of them a few RGB
	// steps apart, and panels and card edges nobody could tell apart. Another step has to be
	// a decision rather than a drift, so the ramps are pinned here.
	it('keeps four text levels, three surfaces and three lines', () => {
		const names = Object.keys(tokens);
		expect(names.filter((name) => name.startsWith('--text-'))).toEqual(TEXT_LEVELS);
		expect(names.filter((name) => name.startsWith('--surface-'))).toEqual(SURFACES);
		expect(names.filter((name) => name.startsWith('--line'))).toEqual([
			'--line',
			'--line-strong',
			'--line-hover'
		]);
	});

	it.each(TEXT)('%s reads at 4.5:1 or better on every surface', (name) => {
		for (const surface of SURFACES) {
			const ratio = contrast(tokens[name], tokens[surface]);
			expect(
				Number(ratio.toFixed(2)),
				`${name} (${tokens[name]}) on ${surface} (${tokens[surface]})`
			).toBeGreaterThanOrEqual(4.5);
		}
	});

	// A chip, a selected card or a warning is a translucent wash over the ground or a panel.
	// The dimmest text level has to survive every one of them, so a component never needs a
	// brighter grey of its own to stay legible on one.
	it('keeps the dimmest text legible on every tinted wash', () => {
		const tints = washes();
		expect(Object.keys(tints).length).toBeGreaterThanOrEqual(5);
		for (const [name, wash] of Object.entries(tints)) {
			for (const surface of ['--surface-0', '--surface-1']) {
				const ground = over(wash, tokens[surface]);
				const ratio = contrast(tokens['--text-muted'], ground);
				expect(
					Number(ratio.toFixed(2)),
					`--text-muted on ${name} over ${surface} (${ground})`
				).toBeGreaterThanOrEqual(4.5);
			}
		}
	});

	it.each(MARKS)('%s stays visible at 3:1 on every surface', (name) => {
		for (const surface of SURFACES) {
			const ratio = contrast(tokens[name], tokens[surface]);
			expect(
				Number(ratio.toFixed(2)),
				`${name} (${tokens[name]}) on ${surface} (${tokens[surface]})`
			).toBeGreaterThanOrEqual(3);
		}
	});

	// --on-accent is the one colour that is read on a filled mint button rather than on a
	// surface, so it is checked against the accent instead.
	it('keeps the filled-button label legible on the accent it sits on', () => {
		expect(contrast(tokens['--on-accent'], tokens['--accent'])).toBeGreaterThanOrEqual(4.5);
	});
});
