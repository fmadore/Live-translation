// The operator's names and terms: speaker names, institutions, acronyms and jargon the speech
// engine is asked to favour for a whole session — Voxtral's `context_bias`, Gemini Transcribe's
// `customVocabulary` and local Whisper's decoder prompt (`providerTakesGlossary`; see
// `docs/mistral-realtime-api.md#glossary`). Part of the session setup, so it is saved with the
// last setup and in meeting profiles, and kept whichever engine is selected. The Rust core
// checks every Start against the same rules and refuses a list that breaks them, so nothing
// here may ever produce one.

/** Bounds on what the list can hold, as the core enforces them. A term's length counts code
 *  points, as Rust's `chars()` does, not UTF-16 units. */
export const MAX_GLOSSARY_TERMS = 100;
export const MAX_GLOSSARY_TERM_LENGTH = 200;

/** Why an entry was not added. A blank entry is not reported: there is nothing to add. */
export type GlossaryProblem = 'blank' | 'length' | 'duplicate' | 'full';

/** Leading and trailing white space, as JavaScript's `trim` sees it and as Rust's does: Rust
 *  also counts U+0085 (next line), which `trim` keeps. */
const EDGE_SPACE = /^[\s\u0085]+|[\s\u0085]+$/gu;

/** The engine is told each spelling once: `Unesco` and `UNESCO` are one entry. */
const termKey = (term: string) => term.toLowerCase();

/** `list` with `input` added at the end, trimmed and composed, or why it was not added. */
export function addGlossaryTerm(
	list: readonly string[],
	input: string
): { list: string[]; term: string } | { problem: GlossaryProblem } {
	const term = input.normalize('NFC').replace(EDGE_SPACE, '');
	if (!term) return { problem: 'blank' };
	if ([...term].length > MAX_GLOSSARY_TERM_LENGTH) return { problem: 'length' };
	if (list.some((entry) => termKey(entry) === termKey(term))) return { problem: 'duplicate' };
	if (list.length >= MAX_GLOSSARY_TERMS) return { problem: 'full' };
	return { list: [...list, term], term };
}

export function removeGlossaryTerm(list: readonly string[], term: string): string[] {
	return list.filter((entry) => termKey(entry) !== termKey(term));
}

/** A valid list from anything: a stored setup or a meeting profile written by another build.
 *  Each acceptable term once, in order, within the limits; the rest are dropped, not the list. */
export function normalizeGlossary(value: unknown): string[] {
	if (!Array.isArray(value)) return [];
	let list: string[] = [];
	for (const entry of value) {
		if (typeof entry !== 'string') continue;
		const added = addGlossaryTerm(list, entry);
		if ('list' in added) list = added.list;
	}
	return list;
}
