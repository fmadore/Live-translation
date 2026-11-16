// The spending guard as it runs: on the session clock, against the stores the session fills.

import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { flushSync } from 'svelte';
import { get } from 'svelte/store';
import { createSpendingGuard, CHECK_EVERY_MS } from './spendingGuard.svelte';
import { costWarningUsd, durationWarningHours, idlePauseMinutes } from './spendingGuard';
import {
	activityTimes,
	applyStatus,
	beginSession,
	options,
	pauseRequested,
	runningSince
} from './stores';
import { DEFAULT_START_OPTIONS } from './startOptions';

const MINUTE = 60_000;
const START = 1_800_000_000_000;

class FakeClock {
	now = $state(START);
	elapsedMs = $state(0);
}
let clock: FakeClock;
let cost: number;
let pause: ReturnType<typeof vi.fn<() => Promise<void>>>;
let stop: (() => void) | undefined;

function start() {
	let guard!: ReturnType<typeof createSpendingGuard>;
	stop = $effect.root(() => {
		guard = createSpendingGuard({ clock, cost: () => cost, pause });
	});
	flushSync();
	return guard;
}

/** Move the clock on, as its once-a-second tick does. */
function advance(ms: number) {
	clock.now += ms;
	clock.elapsedMs += ms;
	flushSync();
}

beforeEach(() => {
	clock = new FakeClock();
	cost = 0;
	pause = vi.fn(async () => pauseRequested.set(true));
	options.set({ ...DEFAULT_START_OPTIONS, mode: 'translate', provider: 'openai' });
	beginSession(get(options));
	applyStatus({ state: 'running', origin: 'system' });
	runningSince.set({ system: START });
	activityTimes.set({});
	idlePauseMinutes.set(10);
	costWarningUsd.set(0);
	durationWarningHours.set(0);
});

afterEach(() => {
	stop?.();
	applyStatus({ state: 'idle' });
	idlePauseMinutes.set(0);
});

it('pauses a cloud session nothing has been captioned in for the chosen minutes', async () => {
	const guard = start();
	advance(9 * MINUTE);
	activityTimes.set({ system: { caption: clock.now, audio: 0, silenced: 0 } });
	advance(9 * MINUTE);
	expect(pause).not.toHaveBeenCalled();
	advance(MINUTE + CHECK_EVERY_MS);
	expect(pause).toHaveBeenCalledOnce();
	expect(guard.idlePaused).toBe(10);
	await Promise.resolve();
	expect(get(pauseRequested)).toBe(true);
	// Paused, it is not paused again.
	advance(30 * MINUTE);
	expect(pause).toHaveBeenCalledOnce();
	// Resumed by any means, the reason goes with the pause.
	pauseRequested.set(false);
	flushSync();
	expect(guard.idlePaused).toBeNull();
});

it('leaves local Whisper and the demo alone: they cost nothing', () => {
	options.update((o) => ({ ...o, mode: 'transcribe', provider: 'whisper' }));
	start();
	advance(60 * MINUTE);
	options.update((o) => ({ ...o, provider: 'ondevice' }));
	advance(60 * MINUTE);
	expect(pause).not.toHaveBeenCalled();
});

it('does nothing while switched off', () => {
	idlePauseMinutes.set(0);
	const guard = start();
	advance(120 * MINUTE);
	expect(pause).not.toHaveBeenCalled();
	expect(guard.idlePaused).toBeNull();
});

it('says nothing about a pause that did not happen', async () => {
	pause = vi.fn(async () => {});
	const guard = start();
	advance(11 * MINUTE);
	expect(guard.idlePaused).toBe(10);
	await Promise.resolve();
	await Promise.resolve();
	expect(guard.idlePaused).toBeNull();
});

it('says once that the estimate and the length have passed, and never stops anything', () => {
	idlePauseMinutes.set(0);
	costWarningUsd.set(5);
	durationWarningHours.set(2);
	const guard = start();
	cost = 4.9;
	advance(119 * MINUTE);
	expect(guard.costPassed).toBeNull();
	expect(guard.durationPassed).toBeNull();
	cost = 5.1;
	advance(MINUTE);
	expect(guard.costPassed).toBe(5);
	expect(guard.durationPassed).toBe(2);
	guard.dismiss('cost');
	guard.dismiss('duration');
	advance(MINUTE);
	// Dismissed is dismissed: said once a session.
	expect(guard.costPassed).toBeNull();
	expect(guard.durationPassed).toBeNull();
	expect(pause).not.toHaveBeenCalled();
	// A new session may say it again.
	applyStatus({ state: 'idle' });
	flushSync();
	applyStatus({ state: 'running', origin: 'system' });
	advance(MINUTE);
	expect(guard.costPassed).toBe(5);
	guard.dismiss('idle');
});
