import { beforeEach, describe, expect, it } from 'vitest';
import { get } from 'svelte/store';
import {
	applyStatus,
	clearTranscript,
	flushTranscript,
	markTranscriptSaved,
	pushCaption,
	restoreTranscript,
	savedPath,
	transcript,
	transcriptDirty
} from './stores';
import { NOTHING_SAVED } from './document';
import type { Caption, TranscriptLine } from './types';

function caption(turnId: number, text: string, final = true): Caption {
	return { turnId, text, sourceText: `src ${text}`, final, origin: 'microphone' };
}

beforeEach(() => {
	clearTranscript();
});

describe('transcript capacity', () => {
	// The whole of issue #25: the log used to be `.slice(0, 1000)`d on every commit, so a long
	// event lost its own beginning without ever saying so.
	it('keeps every line of a session far past the old 1,000-line cap', () => {
		for (let i = 1; i <= 2500; i++) pushCaption(caption(i, `utterance ${i}`));

		const lines = get(transcript);
		expect(lines).toHaveLength(2500);
		// Newest first, so the oldest line is the one truncation used to eat.
		expect(lines[lines.length - 1].text).toBe('utterance 1');
		expect(lines[0].text).toBe('utterance 2500');
	});
});

describe('saved / unsaved state', () => {
	it('starts clean and goes unsaved on the first finalized line', () => {
		expect(get(transcriptDirty)).toBe(false);
		pushCaption(caption(1, 'hello'));
		expect(get(transcriptDirty)).toBe(true);
	});

	it('becomes saved once the log has been written', () => {
		pushCaption(caption(1, 'hello'));
		markTranscriptSaved(get(transcript), 'C:\\Docs\\transcript.md');

		expect(get(transcriptDirty)).toBe(false);
		expect(get(savedPath)).toBe('C:\\Docs\\transcript.md');
	});

	it('stays saved when the same document is saved a second time', () => {
		pushCaption(caption(1, 'hello'));
		markTranscriptSaved(get(transcript), 'C:\\Docs\\transcript.txt');
		markTranscriptSaved(get(transcript), 'C:\\Docs\\transcript.md');

		expect(get(transcriptDirty)).toBe(false);
		expect(get(savedPath)).toBe('C:\\Docs\\transcript.md');
	});

	it('goes unsaved again when the session continues after a save', () => {
		pushCaption(caption(1, 'hello'));
		markTranscriptSaved(get(transcript), 'C:\\Docs\\transcript.md');
		pushCaption(caption(2, 'and more'));

		expect(get(transcriptDirty)).toBe(true);
	});

	// Discarding has to reset the marker with the text. Ids keep climbing, so a stale marker
	// left behind would make the next run's first lines look as if they were already on disk.
	it('forgets the save marker when the log is cleared', () => {
		pushCaption(caption(1, 'hello'));
		markTranscriptSaved(get(transcript), 'C:\\Docs\\transcript.md');
		clearTranscript();

		expect(get(transcript)).toEqual([]);
		expect(get(savedPath)).toBe('');
		expect(get(transcriptDirty)).toBe(false);

		pushCaption(caption(2, 'a fresh session'));
		expect(get(transcriptDirty)).toBe(true);
	});
});

describe('finalizing the document at shutdown', () => {
	// A stop can arrive between a provider's last delta and its turn-complete. That text is
	// the operator's, and quitting must not be what loses it.
	it('commits an in-flight turn when the session ends', () => {
		pushCaption(caption(1, 'half a sentence', false));
		expect(get(transcript)).toHaveLength(0);

		flushTranscript();

		expect(get(transcript).map((l) => l.text)).toEqual(['half a sentence']);
		expect(get(transcriptDirty)).toBe(true);
	});

	it('commits the in-flight turn on a whole-session idle status too', () => {
		pushCaption(caption(1, 'trailing words', false));
		applyStatus({ state: 'idle' });

		expect(get(transcript).map((l) => l.text)).toEqual(['trailing words']);
	});

	it('is safe to flush twice', () => {
		pushCaption(caption(1, 'once only', false));
		flushTranscript();
		flushTranscript();

		expect(get(transcript)).toHaveLength(1);
	});
});

describe('restoring a recovered snapshot', () => {
	const recovered: TranscriptLine[] = [
		{ id: 2, text: 'second', sourceText: 'src second', origin: 'system' },
		{ id: 1, text: 'first', sourceText: 'src first', origin: 'microphone' }
	];

	it('loads the lines back and treats them as unsaved', () => {
		restoreTranscript(recovered);

		expect(get(transcript)).toEqual(recovered);
		expect(get(transcriptDirty)).toBe(true);
		expect(get(savedPath)).toBe('');
	});

	// Restored ids come from the crashed run. Continuing to caption must not mint an id that
	// collides with one of them, or the list keys and the save marker both go wrong.
	it('continues numbering above the recovered lines', () => {
		restoreTranscript(recovered);
		pushCaption(caption(9, 'after the crash'));

		const ids = get(transcript).map((l) => l.id);
		expect(new Set(ids).size).toBe(ids.length);
		expect(Math.max(...ids)).toBeGreaterThan(2);
	});

	it('leaves the marker unset so the very first save covers everything', () => {
		restoreTranscript(recovered);
		expect(get(transcriptDirty)).toBe(true);

		markTranscriptSaved(get(transcript), 'C:\\Docs\\recovered.md');
		expect(get(transcriptDirty)).toBe(false);
		expect(NOTHING_SAVED).toBe(0);
	});
});
