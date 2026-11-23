// The running cost estimate as a ledger: each stream is billed at its own engine's rate for the
// time it streamed. A run starts with every stream on the setup's engine; a stream that ends
// stops adding to the estimate, and one continued by offline Whisper costs nothing from then on
// (#248). Times are elapsed session milliseconds. Each paused track closes its own billed interval.

import { PROVIDER_META } from './providers';
import { LANES, laneCount, trackOf } from './types';
import type { Provider, StartOptions, Track } from './types';

/** A stream changing what it is billed at, `atMs` into the run: another engine from
 *  then on, or `null` when it stopped streaming. */
export interface BillingChange {
	track: Track;
	provider: Provider | null;
	atMs: number;
}

/** What `track` is billed at once every change so far has applied, starting from `initial`. */
export function billedAt(
	track: Track,
	initial: Provider,
	changes: readonly BillingChange[]
): Provider | null {
	let current: Provider | null = initial;
	for (const change of changes) if (change.track === track) current = change.provider;
	return current;
}

/** The estimate for `tracks` after `nowMs` elapsed, in US dollars: each stream's time at each
 *  engine it ran on, at that engine's hourly figure. With no change it is the setup's rate
 *  times the streams, as it always was. */
export function ledgerCost(
	tracks: readonly Track[],
	initial: Provider,
	changes: readonly BillingChange[],
	nowMs: number
): number {
	let usd = 0;
	for (const track of tracks) {
		let provider: Provider | null = initial;
		let from = 0;
		const close = (to: number) => {
			if (provider)
				usd += (PROVIDER_META[provider].hourlyEstimate * Math.max(0, to - from)) / 3_600_000;
		};
		for (const change of changes) {
			if (change.track !== track) continue;
			const at = Math.min(Math.max(change.atMs, from), nowMs);
			close(at);
			provider = change.provider;
			from = at;
		}
		close(nowMs);
	}
	return usd;
}

/** The streams a run opens, as tracks: each source it captures, once per caption language.
 *  Counted as `streamCount` counts them, from the sources the page says the run uses. */
export function runTracks(
	usesMic: boolean,
	usesSystem: boolean,
	options: Pick<StartOptions, 'mode' | 'provider' | 'targetLanguage' | 'secondTargetLanguage'>
): Track[] {
	const origins = [
		...(usesMic ? (['microphone'] as const) : []),
		...(usesSystem ? (['system'] as const) : [])
	];
	const lanes = LANES.slice(0, laneCount(options));
	return origins.flatMap((origin) => lanes.map((lane) => trackOf(origin, lane)));
}
