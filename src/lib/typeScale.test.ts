// The type scale is only as good as its coverage.
//
// A font size written straight into a component as pixels is a size Windows' accessibility
// text setting cannot reach — and it fails silently, because the window still looks right on
// the developer's machine at 100%. That is exactly how the contrast failures behind issue #24
// arrived, one shade at a time, so the size ramp gets the same kind of guard the colour ramp
// has in `palette.test.ts`: read the stylesheets, and fail on anything that opted out.

import { describe, expect, it } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

import { applyTextScale, clampTextScale, TEXT_SCALE_DEFAULT, TEXT_SCALE_MAX } from './textScale';

const SRC = join(process.cwd(), 'src');
const APP_CSS = readFileSync(join(SRC, 'app.css'), 'utf8');

function sourceFiles(dir: string): string[] {
	return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
		const path = join(dir, entry.name);
		if (entry.isDirectory()) return sourceFiles(path);
		return entry.name.endsWith('.svelte') ? [path] : [];
	});
}

/** Every `font-size:` declaration in the app's components, with the file it came from. */
function declarations(): Array<{ file: string; value: string }> {
	return sourceFiles(SRC).flatMap((file) =>
		[...readFileSync(file, 'utf8').matchAll(/font-size:\s*([^;]+);/g)].map((m) => ({
			file: file.slice(SRC.length + 1).replace(/\\/g, '/'),
			value: m[1].trim()
		}))
	);
}

describe('the type scale', () => {
	it('is declared once, in app.css, and every step is a multiple of the factor', () => {
		const steps = [
			...APP_CSS.matchAll(/--type-[\w-]+:\s*calc\(([\d.]+)px \* var\(--text-scale\)\);/g)
		];
		expect(steps.length).toBeGreaterThan(0);
		for (const [, px] of steps) expect(Number(px)).toBeGreaterThan(0);
	});

	it('carries the factor into the root font size, so em-based layout follows the text', () => {
		expect(APP_CSS).toContain('font-size: calc(16px * var(--text-scale));');
	});

	// The overlay is the one exception, and a narrow one: its caption size is a number the
	// operator sets for the room, so those two declarations are expressions over `--fs`, not
	// literals. Anything else with a raw px size is a component that will not grow.
	it('leaves no component declaring its own pixel size', () => {
		const literal = declarations().filter(({ value }) => /^[\d.]+px$/.test(value));
		expect(literal).toEqual([]);
	});

	it('is what every component actually uses', () => {
		const used = declarations().filter(({ file }) => !file.startsWith('routes/overlay/'));
		expect(used.length).toBeGreaterThan(0);
		for (const { file, value } of used) {
			expect(value, `${file} declares ${value}`).toMatch(/var\(--type-[\w-]+\)|inherit/);
		}
	});
});

describe('clampTextScale', () => {
	it('passes the Windows range through untouched', () => {
		for (const factor of [1, 1.25, 1.45, 1.75, 2, 2.25]) {
			expect(clampTextScale(factor)).toBe(factor);
		}
	});

	it('holds anything past the slider at its edge', () => {
		expect(clampTextScale(0.2)).toBe(TEXT_SCALE_DEFAULT);
		expect(clampTextScale(9)).toBe(TEXT_SCALE_MAX);
	});

	// The factor crosses a JSON event boundary, so it can arrive as anything at all. A `NaN`
	// in `--text-scale` would invalidate every `calc()` in the stylesheet at once.
	it('falls back to no scaling for anything that is not a real number', () => {
		for (const junk of [NaN, Infinity, -Infinity, null, undefined, '2', {}]) {
			expect(clampTextScale(junk)).toBe(TEXT_SCALE_DEFAULT);
		}
	});
});

describe('applyTextScale', () => {
	function target() {
		const written: Record<string, string> = {};
		return {
			written,
			style: {
				setProperty: (name: string, value: string) => {
					written[name] = value;
				}
			}
		};
	}

	it('writes the clamped factor where the stylesheet reads it', () => {
		const root = target();
		applyTextScale(2.25, root);
		expect(root.written['--text-scale']).toBe('2.25');
	});

	it('never writes a value that would invalidate the stylesheet', () => {
		const root = target();
		applyTextScale('nonsense', root);
		expect(root.written['--text-scale']).toBe('1');
	});
});
