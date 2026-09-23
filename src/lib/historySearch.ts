import type { SavedSession } from './history';
export interface HistoryFilter {
	query: string;
	from: string;
	to: string;
	language: string;
}

function fold(text: string): string {
	return text.normalize('NFKC').toLocaleLowerCase();
}

// Normalising every line of every session on each keystroke cost tens of milliseconds with a
// long history. A listed session is decoded once and never mutated — a rename re-lists — so
// its folded text is computed on first search and reused until the list is replaced.
const searchText = new WeakMap<SavedSession, string[]>();

function foldedText(session: SavedSession): string[] {
	let texts = searchText.get(session);
	if (!texts) {
		texts = [session.title ?? '', ...session.lines.flatMap((l) => [l.text, l.sourceText])].map(
			fold
		);
		searchText.set(session, texts);
	}
	return texts;
}

/** Date inputs mean the operator's local calendar date, including its whole final day. */
export function matchesSession(session: SavedSession, filter: HistoryFilter): boolean {
	const date = new Date(session.startedAt);
	const day = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
	if ((filter.from && day < filter.from) || (filter.to && day > filter.to)) return false;
	const language = session.targetLanguage ?? session.sourceLanguage;
	if (filter.language && filter.language !== language) return false;
	const query = fold(filter.query.trim());
	if (!query) return true;
	return foldedText(session).some((text) => text.includes(query));
}
