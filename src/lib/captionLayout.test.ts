import { describe, expect, it } from 'vitest';
import {
	appendCaptionHistory,
	CAPTION_TAIL_CHARS,
	captionReach,
	firstOffsetOnLine,
	fitCaptionTail,
	isCaptionLayout,
	bottomCaptionHeight,
	originalHeight,
	rowFontSize,
	fitRows,
	LEAST_ROW_FONT,
	stableTurnTail,
	TIGHT_ROW_GAP,
	tail,
	type RowFit
} from './captionLayout';

describe('row font size', () => {
	it('keeps the operator size when a line of it fits the row', () => {
		expect(rowFontSize(52, 128)).toBe(52);
		expect(rowFontSize(38, 200, true)).toBe(38);
	});

	it('shrinks so a shallow row still shows a line', () => {
		// A 160 px strip at 52 px split between two sources: 55 px per row.
		const size = rowFontSize(52, 55);
		expect(size).toBeLessThan(52);
		expect(Math.floor(55 / (size * 1.34))).toBeGreaterThanOrEqual(1);
		const withOriginal = rowFontSize(52, 90, true);
		expect(withOriginal * 1.34 + originalHeight(withOriginal, 1) + 4).toBeLessThanOrEqual(90);
	});

	it('reports the size that fits, however small, and leaves the floors to fitRows', () => {
		expect(rowFontSize(52, 5)).toBe(3);
		expect(rowFontSize(52, 0)).toBe(0);
	});
});

describe('fitting rows into the window', () => {
	/** One line at the drawn size, and the original under it when it is kept, in each row. */
	function holdsALine(fit: RowFit) {
		const line = fit.font * 1.34 + (fit.original ? originalHeight(fit.font, 1) + 4 : 0);
		return line <= fit.rowHeight;
	}
	// The default strip: 160 px, less 16 px above and below in Fit window.
	const strip = 160 - 32;

	it('keeps the chosen size and the full gap when the rows have room', () => {
		expect(fitRows(38, strip, 1)).toEqual({
			font: 38,
			gap: 18,
			rowHeight: strip,
			original: false,
			room: true
		});
		expect(fitRows(38, 400, 2, true)).toMatchObject({ font: 38, gap: 18, original: true });
	});

	// Both sources in two caption languages used to get 18.5 px rows and a 20 px floor whose
	// line is 26.8 px: four labels and no caption.
	it('gives three and four rows in the default strip a line each, at 20 px or more', () => {
		for (const rows of [3, 4]) {
			const fit = fitRows(38, strip, rows);
			expect(fit.font).toBeGreaterThanOrEqual(20);
			expect(fit.font).toBeLessThan(38);
			expect(fit.gap).toBe(TIGHT_ROW_GAP);
			expect(fit.room).toBe(true);
			expect(holdsALine(fit)).toBe(true);
			expect(fit.rowHeight * rows + fit.gap * (rows - 1)).toBeLessThanOrEqual(strip);
		}
	});

	// With the original on, it showed and the translation did not. The translation comes first.
	it('drops the original before the translation goes below 20 px', () => {
		const four = fitRows(38, strip, 4, true);
		expect(four).toMatchObject({ original: false, room: true });
		expect(four.font).toBeGreaterThanOrEqual(20);
		expect(holdsALine(four)).toBe(true);
		// Two rows keep it, closed up and smaller.
		const two = fitRows(38, strip, 2, true);
		expect(two).toMatchObject({ original: true, gap: TIGHT_ROW_GAP, room: true });
		expect(two.font).toBeGreaterThanOrEqual(20);
		expect(holdsALine(two)).toBe(true);
	});

	it('goes below 20 px, down to the hard floor, rather than show no caption', () => {
		const fit = fitRows(38, 90, 4);
		expect(fit.font).toBeLessThan(20);
		expect(fit.font).toBeGreaterThanOrEqual(LEAST_ROW_FONT);
		expect(fit.room).toBe(true);
		expect(holdsALine(fit)).toBe(true);
	});

	it('says when even the hard floor has no room', () => {
		const fit = fitRows(38, 40, 4);
		expect(fit).toMatchObject({ font: LEAST_ROW_FONT, room: false, original: false });
	});

	it('leaves the full-size strip Snap to bottom makes at the chosen size', () => {
		for (const rows of [1, 2, 3, 4])
			for (const original of [false, true]) {
				const height = bottomCaptionHeight(38, rows, 'fit', original) - 32;
				expect(fitRows(38, height, rows, original)).toMatchObject({
					font: 38,
					gap: 18,
					original
				});
			}
	});
});

it('makes room for one line of original speech per row when it is shown', () => {
	for (const sources of [1, 2]) {
		const plain = bottomCaptionHeight(60, sources, 'fit');
		const bilingual = bottomCaptionHeight(60, sources, 'fit', true);
		expect(bilingual - plain).toBe(sources * (Math.ceil(0.6 * 60 * 1.34) + 4));
	}
});

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
	it('finds the same tail from a bounded search while laying out far less', () => {
		const context = Array.from({ length: 2000 }, (_, i) => `word${i}`).join(' ');
		const probes: number[] = [];
		const fits = (s: string) => (probes.push(s.length), s.length <= 120);
		const unbounded = fitCaptionTail(context, fits);
		const widest = Math.max(...probes);
		probes.length = 0;
		expect(fitCaptionTail(context, fits, 400)).toBe(unbounded);
		expect(Math.max(...probes)).toBeLessThanOrEqual(402);
		expect(widest).toBeGreaterThan(10000);
	});
	it('falls back to the whole text when the bound was too tight', () => {
		expect(fitCaptionTail(text, (s) => s.length <= 55, 10)).toBe(
			fitCaptionTail(text, (s) => s.length <= 55)
		);
		expect(fitCaptionTail(text, () => true, 10)).toBe(text);
		// No space to cut at (Japanese has none): the bound cannot apply.
		expect(fitCaptionTail('あいうえおかきくけこ', (s) => Array.from(s).length <= 5, 3)).toBe(
			'… くけこ'
		);
	});
	it('bounds reach by the region, never below what it could show', () => {
		expect(captionReach(0, 100, 50)).toBe(Infinity);
		expect(captionReach(800, 100, 1)).toBe(Infinity);
		// Two rows of 50px text, 800px wide: even 0.3em glyphs would give 107 a row.
		const reach = captionReach(800, 100, 50);
		expect(reach).toBeGreaterThan(2 * 107);
		expect(reach).toBeLessThan(1000);
	});
	it('retains multiple recent turns with a bounded history', () => {
		expect(appendCaptionHistory(appendCaptionHistory('', 'One.'), 'Two.')).toBe('One. Two.');
		const history = appendCaptionHistory('old '.repeat(5000), 'Newest turn.');
		expect(history.length).toBeLessThanOrEqual(CAPTION_TAIL_CHARS);
		expect(history.endsWith('Newest turn.')).toBe(true);
	});
	it('rejects unsupported persisted layout values', () => {
		expect(isCaptionLayout('fit')).toBe(true);
		expect(isCaptionLayout('compact')).toBe(true);
		expect(isCaptionLayout('wide')).toBe(false);
	});
});

describe('tail', () => {
	it('keeps short text whole and cuts long text at a nearby word boundary', () => {
		expect(tail('  a   short   line ', 40)).toBe('a short line');
		expect(tail('alpha beta gamma delta', 10)).toBe('… delta');
		expect(tail('', 10)).toBe('');
	});

	it('cuts inside a word only when no space is near, and never keeps more than asked', () => {
		const token = 'x'.repeat(60);
		expect(tail(`start ${token}`, 30)).toBe('… ' + 'x'.repeat(30));
		const speech = Array.from({ length: 5000 }, (_, i) => `word${i}`).join(' ');
		const kept = tail(speech, CAPTION_TAIL_CHARS);
		expect(kept.length).toBeLessThanOrEqual(CAPTION_TAIL_CHARS + 2);
		expect(kept.startsWith('… word')).toBe(true);
		expect(kept.endsWith('word4999')).toBe(true);
	});
});

describe('stable reading trim point', () => {
	// Ten characters to a line: offsets 0–9 on line 0, 10–19 on line 1, and so on.
	const lineOf = (offset: number) => Math.floor(offset / 10);

	it('finds the first character laid out on the requested line', () => {
		expect(firstOffsetOnLine(95, lineOf, 3)).toBe(30);
		expect(firstOffsetOnLine(95, lineOf, 9)).toBe(90);
	});

	it('cuts nothing when no text reaches that line, or line zero is asked for', () => {
		expect(firstOffsetOnLine(95, lineOf, 10)).toBe(0);
		expect(firstOffsetOnLine(95, lineOf, 0)).toBe(0);
		expect(firstOffsetOnLine(0, lineOf, 1)).toBe(0);
	});

	it('handles lines of uneven length', () => {
		const starts = [0, 4, 31, 32, 60];
		const uneven = (offset: number) => starts.findLastIndex((start) => offset >= start);
		expect(firstOffsetOnLine(80, uneven, 2)).toBe(31);
		expect(firstOffsetOnLine(80, uneven, 3)).toBe(32);
	});
});

describe('Stable reading’s bound on a long turn', () => {
	it('leaves a turn that fits whole', () => {
		const turn = 'word '.repeat(2000).trim();
		expect(stableTurnTail(turn)).toBe(turn);
	});

	it('keeps the newest part of a long turn from a word start, and moves the cut in steps', () => {
		const words = Array.from({ length: 4000 }, (_, i) => `w${i}`);
		const turn = words.join(' ');
		expect(turn.length).toBeGreaterThan(CAPTION_TAIL_CHARS);
		const kept = stableTurnTail(turn);
		expect(kept.length).toBeLessThanOrEqual(CAPTION_TAIL_CHARS);
		expect(kept.length).toBeGreaterThan(CAPTION_TAIL_CHARS - 2_100);
		expect(turn.endsWith(kept)).toBe(true);
		// From a word start, with nothing in front of it.
		expect(words).toContain(kept.split(' ')[0]);
		// A few more words leave the cut where it was: lines already laid out keep their wrap.
		const longer = `${turn} w4000 w4001 w4002`;
		expect(stableTurnTail(longer)).toBe(`${kept} w4000 w4001 w4002`);
	});
});
