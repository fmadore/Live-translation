import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

// Issue #24. The contrast failures this guards against were not written deliberately — they
// arrived one shade at a time, because nothing in the repo could tell that `--muted-3` had
// drifted under 4.5:1 on the panel it is read on. This is that check: it reads the real
// tokens out of `app.css`, so dimming one fails here rather than on a reviewer's machine.
//
// It covers the token layer only. Text on a tinted wash (a selected engine's mint background,
// a warning chip) composites two colours the stylesheet never names together, and is verified
// against the rendered window instead — see `docs/accessibility.md`.

const CSS = readFileSync(new URL('../app.css', import.meta.url), 'utf8');

/** Every `--name: #rrggbb` in `:root`. Non-hex values (rgba washes) are skipped. */
function palette(): Record<string, string> {
	const root = CSS.slice(CSS.indexOf(':root'), CSS.indexOf('\n}'));
	const out: Record<string, string> = {};
	for (const [, name, hex] of root.matchAll(/(--[\w-]+):\s*(#[0-9a-fA-F]{6})\s*;/g)) {
		out[name] = hex;
	}
	return out;
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

/** Everything a window paints text onto. */
const SURFACES = ['--bg', '--surface-0', '--panel', '--panel-2', '--surface-3'];

/** Everything that carries text, dimmest last. WCAG AA for normal-sized text is 4.5:1. */
const TEXT = [
	'--text',
	'--text-bright',
	'--text-soft',
	'--text-dim',
	'--muted',
	'--muted-2',
	'--muted-3',
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

	it.each(TEXT)('%s reads at 4.5:1 or better on every surface', (name) => {
		for (const surface of SURFACES) {
			const ratio = contrast(tokens[name], tokens[surface]);
			expect(
				Number(ratio.toFixed(2)),
				`${name} (${tokens[name]}) on ${surface} (${tokens[surface]})`
			).toBeGreaterThanOrEqual(4.5);
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
