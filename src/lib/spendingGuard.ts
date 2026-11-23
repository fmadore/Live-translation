// The spending guard for cloud engines: a session forgotten over lunch bills every stream it
// has open. Pure functions of times and amounts, so the rules are tested without a clock; the
// operator window drives them from its once-a-second session clock (`spendingGuard.svelte.ts`).
//
// Three choices, each off unless the operator picks a value from its list:
// - pause once nothing has been captioned for so many minutes, through the ordinary Pause;
// - say once that the running cost estimate has passed an amount;
// - say once that the session has run for so many hours.
// None of them stops a session or touches the transcript.

import { persisted, readStored } from './persisted';
import { providerRequiresKey } from './providers';
import type { Origin, Provider, SessionState, Track } from './types';

/** Minutes without a caption before a cloud session is paused. 0 is off. */
export const IDLE_PAUSE_CHOICES = [0, 5, 10, 15, 30] as const;
/** US dollars of estimated cost before the operator is told. 0 is off. */
export const COST_WARNING_CHOICES = [0, 1, 2, 5, 10, 20, 50] as const;
/** Hours of session before the operator is told. 0 is off. */
export const DURATION_WARNING_CHOICES = [0, 1, 2, 3, 4, 6, 8] as const;

const IDLE_PAUSE_KEY = 'guard.idlePauseMinutes';
const COST_WARNING_KEY = 'guard.costWarningUsd';
const DURATION_WARNING_KEY = 'guard.durationWarningHours';

/** A stored choice, or 0 (off) for anything that is not one of `choices`. */
export function readChoice(key: string, choices: readonly number[]): number {
	const value = Number(readStored(key));
	return choices.includes(value) ? value : 0;
}

export const idlePauseMinutes = persisted(IDLE_PAUSE_KEY, () =>
	readChoice(IDLE_PAUSE_KEY, IDLE_PAUSE_CHOICES)
);
export const costWarningUsd = persisted(COST_WARNING_KEY, () =>
	readChoice(COST_WARNING_KEY, COST_WARNING_CHOICES)
);
export const durationWarningHours = persisted(DURATION_WARNING_KEY, () =>
	readChoice(DURATION_WARNING_KEY, DURATION_WARNING_CHOICES)
);

/** Whether time on this engine is billed: the cloud engines are, local Whisper and the
 *  built-in demonstration are not — the same line as whether it needs a key. */
export function isBilled(provider: Provider): boolean {
	return providerRequiresKey(provider);
}

/** Whether anything the run still streams is billed: a stream on the setup's engine or on
 *  another it was continued with (#248), while it is active. A run whose cloud streams have
 *  all ended, or been continued by offline Whisper, has nothing left for the guard to watch. */
export function billedStreamOpen(
	states: Partial<Record<Track, SessionState>>,
	engines: Partial<Record<Track, Provider>>,
	provider: Provider
): boolean {
	return (Object.keys(states) as Track[]).some(
		(track) =>
			states[track] !== 'error' && states[track] !== 'idle' && isBilled(engines[track] ?? provider)
	);
}

/**
 * The last sign of life a session gave: its newest caption from any source, or the moment a
 * source last came to be running (the start, a Resume, a reconnect), whichever is later. A
 * Resume therefore restarts the wait, and a session that has just begun is not idle yet.
 */
export function lastSignOfLife(
	captions: Partial<Record<Origin, { caption?: number }>>,
	runningSince: Partial<Record<Origin, number>>
): number {
	const times = [
		...Object.values(captions).map((activity) => activity?.caption ?? 0),
		...Object.values(runningSince).map((since) => since ?? 0)
	];
	return times.length ? Math.max(...times) : 0;
}

/** Whether a running, billed session that last gave a sign of life at `since` should be paused
 *  now. `minutes` 0 is off; no sign of life at all (`since` 0) is never idle. */
export function idlePauseDue(now: number, since: number, minutes: number): boolean {
	return minutes > 0 && since > 0 && now - since >= minutes * 60_000;
}

/** Whether a cost estimate has reached the threshold, 0 being off. */
export function costWarningDue(cost: number, threshold: number): boolean {
	return threshold > 0 && cost >= threshold;
}

/** Whether a session `elapsedMs` long has reached `hours`, 0 being off. */
export function durationWarningDue(elapsedMs: number, hours: number): boolean {
	return hours > 0 && elapsedMs >= hours * 3_600_000;
}
