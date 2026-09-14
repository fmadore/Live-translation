import { describe, expect, it } from 'vitest';
import {
	appendCaptionHistory,
	fitCaptionTail,
	isCaptionLayout,
	bottomCaptionHeight
} from './captionLayout';

it('bottom-aligns to two lines per visible source instead of retaining a tall window', () => {
	expect(bottomCaptionHeight(38, 0, 'fit')).toBe(160);
	expect(bottomCaptionHeight(38, 1, 'fit')).toBe(160);
	for (const layout of ['fit', 'compact'] as const) {
		for (const font of [20, 38, 96]) {
			for (const sources of [1, 2]) {
				const height = bottomCaptionHeight(font, sources, layout);
				const available = (layout === 'fit' ? height - 32 : height * 0.85) - 18 * (sources - 1);
				expect(available / sources).toBeGreaterThanOrEqual(2 * font * 1.34);
				if (font >= 38) expect(available / sources).toBeLessThan(3 * font * 1.34);
			}
		}
	}
});

describe('caption fitting', () => {
	const text = 'First sentence provides context. The newest words must remain visible.';
	it('reveals more context as the available region grows', () => {
		const small = fitCaptionTail(text, (s) => s.length <= 30);
		const large = fitCaptionTail(text, (s) => s.length <= 55);
		expect(small).toBe('… words must remain visible.');
		expect(large.length).toBeGreaterThan(small.length);
		expect(text.endsWith(large.slice(2))).toBe(true);
		expect(fitCaptionTail(text, () => true)).toBe(text);
	});
	it('handles unbroken tokens and regions too short for a line', () => {
		expect(fitCaptionTail('abcdefghijklmnop', (s) => s.length <= 8)).toBe('… klmnop');
		expect(fitCaptionTail(text, () => false)).toBe('');
		expect(fitCaptionTail('  ', () => false)).toBe('');
	});
	it('normalizes streaming whitespace and preserves Unicode when shortening a token', () => {
		expect(fitCaptionTail(' bonjour\n le   monde ', () => true)).toBe('bonjour le monde');
		expect(fitCaptionTail('😀😀😀😀😀😀', (s) => Array.from(s).length <= 4)).toBe('… 😀😀');
	});
	it('retains multiple recent turns with a bounded history', () => {
		expect(appendCaptionHistory(appendCaptionHistory('', 'One.'), 'Two.')).toBe('One. Two.');
		const history = appendCaptionHistory('old '.repeat(5000), 'Newest turn.');
		expect(history.length).toBeLessThanOrEqual(12000);
		expect(history.endsWith('Newest turn.')).toBe(true);
	});
	it('rejects unsupported persisted layout values', () => {
		expect(isCaptionLayout('fit')).toBe(true);
		expect(isCaptionLayout('compact')).toBe(true);
		expect(isCaptionLayout('wide')).toBe(false);
	});
});
