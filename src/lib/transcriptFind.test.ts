// Find in the live transcript: History's matching, placed back onto the text on screen, and the
// way the current match is kept as the log grows.

import { describe, expect, it } from 'vitest';
import { findMatches, fold, foldPlaced } from './historySearch';
import { createFindIndex, keepCurrent, stepMatch, type FindMatch } from './transcriptFind';

const found = (text: string, query: string) =>
	findMatches(foldPlaced(text), query).map(([start, end]) => text.slice(start, end));

describe('matching, as History matches', () => {
	it('ignores case, and finds every occurrence left to right', () => {
		expect(found('Revenue grew. REVENUE fell, revenue held.', 'revenue')).toEqual([
			'Revenue',
			'REVENUE',
			'revenue'
		]);
		expect(found('aaaa', 'aa')).toEqual(['aa', 'aa']);
	});

	it('finds nothing for an empty or blank query', () => {
		expect(found('anything', '')).toEqual([]);
		expect(found('anything', '   ')).toEqual([]);
	});

	it('is the same comparison as the History search', () => {
		expect(fold('Ｆｉｎａｎｃｅ')).toBe('finance');
		expect(found('Ｆｉｎａｎｃｅ report', 'finance')).toEqual(['Ｆｉｎａｎｃｅ']);
	});

	it('places a match on whole characters when folding changes the text’s length', () => {
		// A ligature becomes two letters; an accent typed apart joins its letter.
		expect(found('The ﬁnal ﬁgure', 'fi')).toEqual(['ﬁ', 'ﬁ']);
		expect(found('Café au lait, café noir', 'café')).toEqual(['Café', 'café']);
		expect(found('İstanbul', 'i')).toEqual(['İ']);
	});
});

describe('the find index', () => {
	it('searches every piece in order, and keeps a piece folded while it is unchanged', () => {
		const index = createFindIndex();
		const targets = [
			{ key: 'p1', text: 'The budget' },
			{ key: 'm2', text: 'Budget talk' },
			{ key: 'p3', text: 'no match here' }
		];
		expect(index.search(targets, 'budget')).toEqual<FindMatch[]>([
			{ key: 'p1', start: 4, end: 10 },
			{ key: 'm2', start: 0, end: 6 }
		]);
		// A paragraph that grew is folded again.
		targets[2] = { key: 'p3', text: 'no match here, budget' };
		expect(index.search(targets, 'budget')).toHaveLength(3);
		expect(index.search(targets, ' ')).toEqual([]);
	});
});

describe('the current match', () => {
	const order = (key: string) => Number(key.slice(1));
	const m = (key: string, start: number): FindMatch => ({ key, start, end: start + 1 });

	it('stays on the match the operator was on when new lines arrive', () => {
		const before = [m('p1', 0), m('p4', 2)];
		const after = [m('p1', 0), m('p4', 2), m('p9', 0)];
		expect(keepCurrent(before[1], after, order)).toBe(1);
	});

	it('moves to the next one after it when that match has gone, never back to the top', () => {
		expect(keepCurrent(m('p4', 2), [m('p1', 0), m('p4', 6), m('p9', 0)], order)).toBe(1);
		expect(keepCurrent(m('p4', 2), [m('p1', 0), m('p9', 0)], order)).toBe(1);
		expect(keepCurrent(m('p9', 2), [m('p1', 0), m('p4', 0)], order)).toBe(1);
	});

	it('starts on the first match, and has none to be on when nothing matches', () => {
		expect(keepCurrent(undefined, [m('p1', 0)], order)).toBe(0);
		expect(keepCurrent(m('p1', 0), [], order)).toBe(-1);
	});

	it('steps through the matches and wraps at either end', () => {
		expect(stepMatch(0, 3, 1)).toBe(1);
		expect(stepMatch(2, 3, 1)).toBe(0);
		expect(stepMatch(0, 3, -1)).toBe(2);
		expect(stepMatch(-1, 3, 1)).toBe(0);
		expect(stepMatch(-1, 3, -1)).toBe(2);
		expect(stepMatch(0, 0, 1)).toBe(-1);
	});
});
