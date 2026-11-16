// Find in the live transcript: which pieces of the reader's text match, and where. Matching is
// History's (`historySearch.ts`): the same folding of case and compatibility forms, placed back
// onto the text on screen so the matches can be highlighted there.
//
// It runs when the query changes (after a short pause in typing) and when the log gains a
// line, never per caption delta: the turns still being spoken are not searched, and are found
// once they are final. A piece of text is folded once and kept while it is unchanged, so a long
// transcript costs one pass the first time and a lookup per paragraph after that.

import { findMatches, foldPlaced, type FoldedText } from './historySearch';

/** One piece of searchable text on screen: a paragraph's caption text, the original speech
 *  under it, or a marker's title. `key` names the element it is drawn in. */
export interface FindTarget {
	key: string;
	text: string;
}

/** One match: which piece, and where in its text. */
export interface FindMatch {
	key: string;
	start: number;
	end: number;
}

/** Search `targets` for `query`, in their order. */
export function createFindIndex() {
	let folded = new Map<string, { text: string; placed: FoldedText }>();
	return {
		search(targets: readonly FindTarget[], query: string): FindMatch[] {
			if (!query.trim()) return [];
			const next = new Map<string, { text: string; placed: FoldedText }>();
			const matches: FindMatch[] = [];
			for (const { key, text } of targets) {
				let entry = folded.get(key);
				if (!entry || entry.text !== text) entry = { text, placed: foldPlaced(text) };
				next.set(key, entry);
				for (const [start, end] of findMatches(entry.placed, query))
					matches.push({ key, start, end });
			}
			// Only what is still on screen is kept, so the cache never outgrows the log.
			folded = next;
			return matches;
		}
	};
}

/** The match to stay on after the matches change (a new line, a longer query): the one the
 *  operator was on if it is still there, otherwise the first one at or after where it was —
 *  never a jump back to the top. -1 when there are no matches. */
export function keepCurrent(
	previous: FindMatch | undefined,
	matches: readonly FindMatch[],
	order: (key: string) => number
): number {
	if (!matches.length) return -1;
	if (!previous) return 0;
	const same = matches.findIndex((m) => m.key === previous.key && m.start === previous.start);
	if (same !== -1) return same;
	const at = order(previous.key);
	const after = matches.findIndex(
		(m) => order(m.key) > at || (m.key === previous.key && m.start >= previous.start)
	);
	return after === -1 ? matches.length - 1 : after;
}

/** The next (`step` 1) or previous (`step` -1) match, wrapping at either end. */
export function stepMatch(current: number, count: number, step: 1 | -1): number {
	if (count <= 0) return -1;
	if (current < 0) return step === 1 ? 0 : count - 1;
	return (current + step + count) % count;
}
