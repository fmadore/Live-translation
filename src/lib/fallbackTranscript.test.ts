// A source continued by offline Whisper (#248) writes in the spoken language when Whisper
// cannot translate into the caption language: its lines must not be noted as the caption
// language, or the operator window would set them in the wrong `lang` and direction.

import { afterEach, beforeEach, expect, it } from 'vitest';
import { get } from 'svelte/store';
import {
	applyStatus,
	beginSession,
	clearTranscript,
	loggedLanguage,
	options,
	pushCaption,
	trackEngines,
	trackLanguage,
	transcript
} from './stores';
import { DEFAULT_START_OPTIONS } from './startOptions';
import type { Caption, StartOptions } from './types';

const french: StartOptions = {
	...DEFAULT_START_OPTIONS,
	mode: 'translate',
	provider: 'gemini',
	targetLanguage: 'fr',
	source: 'both'
};

function line(origin: Caption['origin'], turnId: number, text: string): Caption {
	return { origin, turnId, text, sourceText: '', final: true, startMs: 0, endMs: 500 };
}

beforeEach(() => {
	options.set({ ...french });
	beginSession(french);
});
afterEach(() => {
	applyStatus({ state: 'idle' });
	clearTranscript();
	options.set({ ...DEFAULT_START_OPTIONS });
});

it('notes a line after a switch to subtitles in no language, and the other source as before', () => {
	pushCaption(line('system', 1, 'Avant la panne'));
	trackEngines.set({ system: 'whisper' });
	pushCaption(line('system', 3, 'Nach dem Wechsel'));
	pushCaption(line('microphone', 1, 'Toujours traduit'));
	const [microphone, after, before] = get(transcript);
	expect(loggedLanguage(before)).toBe('fr');
	expect(loggedLanguage(after)).toBeUndefined();
	expect(loggedLanguage(microphone)).toBe('fr');
});

it('keeps English for a source Whisper goes on translating into English', () => {
	const english = { ...french, targetLanguage: 'en' as const };
	expect(trackLanguage(english, 'system', { system: 'whisper-translate' })).toBe('en');
	expect(trackLanguage(english, 'system', { system: 'whisper' })).toBeUndefined();
	expect(trackLanguage(english, 'microphone', { system: 'whisper' })).toBe('en');
});

it('forgets the engines when the next run begins', () => {
	trackEngines.set({ system: 'whisper' });
	beginSession(french);
	expect(get(trackEngines)).toEqual({});
});
