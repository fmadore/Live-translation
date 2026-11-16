import type { SavedSession } from './history';
import type { TargetLanguage } from './languages';
export interface HistoryFilter {
	query: string;
	from: string;
	to: string;
	language: string;
}

/** Text as search compares it: compatibility-normalised (NFKC) and lower-cased. */
export function fold(text: string): string {
	return text.normalize('NFKC').toLocaleLowerCase();
}

/** A base character with the combining marks after it: the unit `fold` is applied to when
 *  matches have to be placed in the original text, since normalising can join a letter and its
 *  accent into one character, or split one into several. */
const CLUSTER = /\P{M}\p{M}*|\p{M}+/gu;

/** `text` folded, with where in `text` each folded character came from. */
export interface FoldedText {
	folded: string;
	/** For each folded character, the offset in `text` of the cluster it came from. Absent when
	 *  folding kept every character in place, as it does for almost all text. */
	starts?: number[];
	/** For each folded character, the offset in `text` just past its cluster. */
	ends?: number[];
}

/** Fold `text` so a match in the folded text maps back onto whole characters of the original:
 *  the same comparison as `matchesSession`, placed. Text that folding leaves in place — already
 *  normalised, and lower-cased without changing length, which is nearly all of it — is folded
 *  once; anything else cluster by cluster, so a letter and its accent stay together. */
export function foldPlaced(text: string): FoldedText {
	const whole = fold(text);
	if (whole.length === text.length && text.normalize('NFKC') === text) return { folded: whole };
	let folded = '';
	const starts: number[] = [];
	const ends: number[] = [];
	for (const match of text.matchAll(CLUSTER)) {
		const piece = fold(match[0]);
		const start = match.index;
		const end = start + match[0].length;
		for (let i = 0; i < piece.length; i++) {
			starts.push(start);
			ends.push(end);
		}
		folded += piece;
	}
	return { folded, starts, ends };
}

/** Every place `query` occurs in the text `placed` was folded from, as `[start, end)` offsets
 *  into that text, left to right and not overlapping. Case and compatibility forms are
 *  ignored as the History search ignores them; an empty query finds nothing. */
export function findMatches(placed: FoldedText, query: string): [number, number][] {
	const needle = fold(query.trim());
	if (!needle) return [];
	const { folded, starts, ends } = placed;
	const found: [number, number][] = [];
	let from = 0;
	for (;;) {
		const at = folded.indexOf(needle, from);
		if (at === -1) return found;
		const last = at + needle.length - 1;
		found.push(starts && ends ? [starts[at], ends[last]] : [at, last + 1]);
		from = at + needle.length;
	}
}

// Normalising every line of every session on each keystroke cost tens of milliseconds with a
// long history. A listed session is decoded once and never mutated, and the History tab keeps
// the same object until its file changes (`createHistoryCache`), so its folded text is
// computed on first search and reused for as long as the session is unchanged.
const searchText = new WeakMap<SavedSession, string[]>();

function foldedText(session: SavedSession): string[] {
	let texts = searchText.get(session);
	if (!texts) {
		texts = [
			session.title ?? '',
			...session.lines.flatMap((l) => [l.text, l.sourceText]),
			...(session.markers ?? []).map((m) => m.label)
		].map(fold);
		searchText.set(session, texts);
	}
	return texts;
}

/** The languages a session was captioned in: its target and, when it had one, the second
 *  caption language. Empty for same-language subtitles. Sessions saved before the second
 *  language existed simply have none. */
export function captionLanguagesOf(session: SavedSession): TargetLanguage[] {
	if (!session.targetLanguage) return [];
	return session.secondTargetLanguage
		? [session.targetLanguage, session.secondTargetLanguage]
		: [session.targetLanguage];
}

/** Date inputs mean the operator's local calendar date, including its whole final day. */
export function matchesSession(session: SavedSession, filter: HistoryFilter): boolean {
	const date = new Date(session.startedAt);
	const day = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
	if ((filter.from && day < filter.from) || (filter.to && day > filter.to)) return false;
	// A session in two caption languages was read in both, so either one finds it. Subtitles
	// are in the language spoken, when it is known.
	const captioned = captionLanguagesOf(session);
	const languages: string[] = captioned.length ? captioned : [session.sourceLanguage];
	if (filter.language && !languages.includes(filter.language)) return false;
	const query = fold(filter.query.trim());
	if (!query) return true;
	return foldedText(session).some((text) => text.includes(query));
}
