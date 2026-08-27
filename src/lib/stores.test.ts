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
	return {
		turnId,
		text,
		sourceText: `src ${text}`,
		final,
		origin: 'microphone',
		// One cue per turn, a second long, starting a turn apart.
		startMs: turnId * 1000,
		endMs: turnId * 1000 + 900
	};
}

beforeEach(() => {
	clearTranscript();
});

describe('caption timing', () => {
	// The core stamps the cue; the store's only job is not to lose it on the way to the log.
	it('commits the cue the core stamped', () => {
		pushCaption(caption(3, 'a finished turn'));

		const [line] = get(transcript);
		expect(line.startMs).toBe(3000);
		expect(line.endMs).toBe(3900);
	});

	// An interim is replaced by the final of the same turn, and it is the final's interval
	// that describes the whole cue — the interim's end was only where the provider had got to.
	it('keeps the interval of the caption it actually committed', () => {
		pushCaption({ ...caption(1, 'partial', false), endMs: 1200 });
		pushCaption({ ...caption(1, 'partial then complete'), endMs: 1850 });

		const lines = get(transcript);
		expect(lines).toHaveLength(1);
		expect(lines[0]).toMatchObject({ text: 'partial then complete', startMs: 1000, endMs: 1850 });
	});

	// Two sources, one clock. Interleaving them in one document is only meaningful if their
	// offsets are comparable, which is why the session hands both the same `SessionClock`.
	it('puts both origins on one timeline', () => {
		pushCaption({ ...caption(1, 'from the room'), origin: 'microphone', startMs: 2000 });
		pushCaption({ ...caption(1, 'from the call'), origin: 'system', startMs: 2400 });

		const byOrigin = Object.fromEntries(get(transcript).map((l) => [l.origin, l.startMs]));
		expect(byOrigin.microphone).toBe(2000);
		expect(byOrigin.system).toBe(2400);
	});
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
