import type { SavedSession } from './history';
export interface HistoryFilter {
	query: string;
	from: string;
	to: string;
	language: string;
}
/** Date inputs mean the operator's local calendar date, including its whole final day. */
export function matchesSession(session: SavedSession, filter: HistoryFilter): boolean {
	const date = new Date(session.startedAt);
	const day = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
	if ((filter.from && day < filter.from) || (filter.to && day > filter.to)) return false;
	const language = session.targetLanguage ?? session.sourceLanguage;
	if (filter.language && filter.language !== language) return false;
	const query = filter.query.trim().normalize('NFKC').toLocaleLowerCase();
	if (!query) return true;
	return [session.title ?? '', ...session.lines.flatMap((l) => [l.text, l.sourceText])].some(
		(text) => text.normalize('NFKC').toLocaleLowerCase().includes(query)
	);
}
