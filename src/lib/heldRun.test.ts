// The held run (#248, decision 1): when the last source ends on a failure offline Whisper can
// take over from, the clock and the history record stay open while the banner offers it.
// Continuing resumes the run; declining ends it at the failure's time.

import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import {
	applyStatus,
	beginSession,
	billingChanges,
	clearTranscript,
	endHeldRun,
	heldSince,
	isRunning,
	options,
	pushCaption,
	sessionStartedAt
} from './stores';
import { decodeSession, historyEnabled, sessionHistory } from './history';
import { api } from './tauri';
import { DEFAULT_START_OPTIONS } from './startOptions';
import type { StatusUpdate } from './types';

const STARTED = Date.parse('2026-11-23T10:00:00Z');
const FAILED = STARTED + 60_000;
const quota: StatusUpdate = {
	state: 'error',
	origin: 'system',
	message: { id: 'error.providerStopped', detail: 'Fake — quota' }
};

let files: Map<string, string>;

beforeEach(() => {
	vi.useFakeTimers({ toFake: ['Date'] });
	vi.setSystemTime(STARTED);
	files = new Map();
	vi.spyOn(api, 'writeHistory').mockImplementation(async (id, raw) => {
		files.set(id, raw);
	});
	vi.spyOn(api, 'appendHistory').mockImplementation(async (id, raw) => {
		files.set(id, (files.get(id) ?? '') + raw);
	});
	historyEnabled.set(true);
	options.set({
		...DEFAULT_START_OPTIONS,
		mode: 'translate',
		provider: 'gemini',
		source: 'system'
	});
	beginSession();
	applyStatus({ state: 'running', origin: 'system' });
	pushCaption({
		turnId: 0,
		text: 'Before the failure',
		sourceText: '',
		origin: 'system',
		final: true,
		startMs: 0,
		endMs: 1000
	});
	vi.setSystemTime(FAILED);
});

afterEach(async () => {
	applyStatus({ state: 'idle' });
	await sessionHistory.flush();
	historyEnabled.set(false);
	clearTranscript();
	options.set({ ...DEFAULT_START_OPTIONS });
	vi.useRealTimers();
	vi.restoreAllMocks();
});

/** This run's record. The previous test's run may still append its end here, as a fragment
 *  with no header of its own, which reads as nothing. */
async function record() {
	await sessionHistory.flush();
	const records = [...files].map(([id, raw]) => decodeSession(raw, id)).filter((r) => r !== null);
	expect(records).toHaveLength(1);
	return records[0];
}

it('holds a run whose last source ended on a failure Whisper can take over from', async () => {
	applyStatus(quota);
	expect(get(heldSince)).toBe(FAILED);
	expect(get(isRunning)).toBe(true);
	expect(get(sessionStartedAt)).toBe(STARTED);
	expect((await record()).endedAt).toBeNull();
	// The ended stream stops adding to the estimate.
	expect(get(billingChanges)).toEqual([{ track: 'system', provider: null, atMs: 60_000 }]);
});

it('closes the record at the failure’s time when the operator declines', async () => {
	applyStatus(quota);
	vi.setSystemTime(FAILED + 5 * 60_000);
	endHeldRun();
	expect(get(isRunning)).toBe(false);
	expect(get(sessionStartedAt)).toBeNull();
	expect((await record()).endedAt).toBe(new Date(FAILED).toISOString());
});

it('closes the record at the failure’s time when the operator stops the held run', async () => {
	applyStatus(quota);
	vi.setSystemTime(FAILED + 5 * 60_000);
	applyStatus({ state: 'idle' });
	expect(get(heldSince)).toBeNull();
	expect((await record()).endedAt).toBe(new Date(FAILED).toISOString());
});

it('resumes the run, clock and record, when the replacement reports', async () => {
	applyStatus(quota);
	vi.setSystemTime(FAILED + 10_000);
	applyStatus({ state: 'running', origin: 'system' });
	expect(get(heldSince)).toBeNull();
	expect(get(isRunning)).toBe(true);
	expect(get(sessionStartedAt)).toBe(STARTED);
	expect((await record()).endedAt).toBeNull();
});

it('ends a run whose last source died some other way, as before', async () => {
	applyStatus({ state: 'error', origin: 'system', message: { id: 'error.systemCapture' } });
	expect(get(heldSince)).toBeNull();
	expect(get(isRunning)).toBe(false);
	expect((await record()).endedAt).toBe(new Date(FAILED).toISOString());
});
