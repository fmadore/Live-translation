// A notice on the overlay for a break: what it may hold, and when the captions take over again.

import { describe, expect, it } from 'vitest';
import {
	NO_NOTICE,
	NOTICE_MAX_CHARS,
	noticeLocale,
	noticeOnCaption,
	noticeOnState,
	normalizeNotice,
	showNotice
} from './overlayNotice';
import { fitNoticeFont, LEAST_ROW_FONT } from './captionLayout';

describe('what a notice may hold', () => {
	it('is one line of plain text, bounded', () => {
		expect(normalizeNotice('  Back\nsoon \u0000')).toBe('Back soon');
		expect(normalizeNotice('<b>Back</b> soon')).toBe('<b>Back</b> soon');
		expect(Array.from(normalizeNotice('x'.repeat(200)))).toHaveLength(NOTICE_MAX_CHARS);
	});

	it('is nothing for anything that is not text', () => {
		for (const value of [undefined, null, 42, { text: 'Back soon' }, ['Back soon']])
			expect(normalizeNotice(value)).toBe('');
	});
});

describe('when the captions take over again', () => {
	it('is the first caption after a pause, when the operator leaves that on', () => {
		const up = showNotice('Back soon', 'running');
		expect(up).toEqual({ text: 'Back soon', armed: false });
		// People are still talking: a caption leaves it up.
		expect(noticeOnCaption(up, true)).toBe(up);
		const paused = noticeOnState(up, 'paused');
		expect(paused.armed).toBe(true);
		expect(noticeOnState(paused, 'running')).toBe(paused);
		expect(noticeOnCaption(paused, true)).toBe(NO_NOTICE);
	});

	it('is the first caption at all, for a notice shown while paused', () => {
		const up = showNotice('Captions paused', 'paused');
		expect(up.armed).toBe(true);
		expect(noticeOnCaption(up, true)).toBe(NO_NOTICE);
	});

	it('is never, when the operator switched that off: only Clear takes it down', () => {
		const up = showNotice('Short break', 'paused');
		expect(noticeOnCaption(up, false)).toBe(up);
	});

	it('changes nothing while there is no notice', () => {
		expect(noticeOnState(NO_NOTICE, 'paused')).toBe(NO_NOTICE);
		expect(noticeOnCaption(NO_NOTICE, true)).toBe(NO_NOTICE);
		expect(showNotice('  ', 'running')).toBe(NO_NOTICE);
	});
});

describe('the suggestions', () => {
	it('are in the caption language when there is a catalog for it, else the interface’s', () => {
		expect(noticeLocale('de', 'en')).toBe('de');
		expect(noticeLocale('fr', 'de')).toBe('fr');
		expect(noticeLocale('ja', 'fr')).toBe('fr');
		expect(noticeLocale(undefined, 'de')).toBe('de');
	});
});

describe('the size a notice is set at', () => {
	/** A notice that fits from `limit` px down. */
	const fitsAt = (limit: number) => (font: number) => font <= limit;

	it('is the row’s size when it fits there', () => {
		expect(fitNoticeFont(38, fitsAt(100))).toBe(38);
	});

	it('is the largest size that fits, when the strip is shallow or the text long', () => {
		for (const limit of [13, 20, 27, 37]) expect(fitNoticeFont(38, fitsAt(limit))).toBe(limit);
	});

	it('is the hard floor when nothing fits, and never above the row’s size', () => {
		expect(fitNoticeFont(38, () => false)).toBe(LEAST_ROW_FONT);
		expect(fitNoticeFont(10, () => false)).toBe(10);
	});
});
