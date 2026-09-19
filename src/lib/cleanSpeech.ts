export const CLEAN_SPEECH_KEY = 'overlay.cleanSpeech';

export function loadCleanSpeech(): boolean {
	return typeof localStorage !== 'undefined' && localStorage.getItem(CLEAN_SPEECH_KEY) === 'true';
}

/** Display-only. Exact hesitation tokens, never substrings or meaningful discourse words.
 * Keep uppercase abbreviations and quoted/compound words. Require a completed token while
 * streaming so an incoming "um" does not disappear before becoming "umbrella". */
export function cleanSpeech(text: string, final = true): string {
	let changed = false;
	let leading = false;
	const cleaned = text.replace(
		/(^|[,;:]?\s+|[,;:])(um|uh|erm|hmm|euh|heu)(?=$|[\s,;:.!?])([,;:]?)/giu,
		(match, before: string, token: string, after: string, offset: number) => {
			if (
				token === token.toUpperCase() ||
				(!final && offset + match.length === text.length && !after)
			)
				return match;
			changed = true;
			if (!text.slice(0, offset).trim()) leading = true;
			return before.includes(',') || before.includes(';') || before.includes(':') ? ' ' : before;
		}
	);
	if (!changed) return text;
	const result = cleaned
		.replace(/\s+([,.!?;:])/g, '$1')
		.replace(/\s+/g, ' ')
		.replace(/^[,;:.!?\s]+/, '')
		.trim();
	return leading ? result.replace(/^\p{Ll}/u, (c) => c.toUpperCase()) : result;
}
