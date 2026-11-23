// The spending guard as it runs (`spendingGuard.ts` holds the rules): it looks at the session
// on the operator window's own clock, which already ticks once a second while a session is
// open, and acts at most every `CHECK_EVERY_MS` — no timer of its own, and nothing per caption.
// What it read about captions is a timestamp the transcript pipeline already records
// (`noteActivity`), so a caption costs it nothing.

import { fromStore, get } from 'svelte/store';
import { untrack } from 'svelte';
import {
	activityTimes,
	isRunning,
	options,
	originStates,
	pauseRequested,
	runningSince,
	trackEngines
} from './stores';
import {
	billedStreamOpen,
	costWarningDue,
	costWarningUsd,
	durationWarningDue,
	durationWarningHours,
	idlePauseDue,
	idlePauseMinutes,
	lastSignOfLife
} from './spendingGuard';

/** How often the guard looks: coarse on purpose, since every rule is counted in minutes. */
export const CHECK_EVERY_MS = 15_000;

export type GuardNotice = 'idle' | 'cost' | 'duration';

export function createSpendingGuard({
	clock,
	cost,
	pause
}: {
	/** The session clock: `now` ticks while a session is open. */
	clock: { readonly now: number; readonly elapsedMs: number };
	/** The running cost estimate, in US dollars, as the live rail shows it. */
	cost: () => number;
	/** Pause the session, through the same path as the Pause button. */
	pause: () => Promise<void>;
}) {
	const running = fromStore(isRunning);
	const paused = fromStore(pauseRequested);
	const setup = fromStore(options);
	const minutes = fromStore(idlePauseMinutes);
	const threshold = fromStore(costWarningUsd);
	const hours = fromStore(durationWarningHours);

	/** The minutes the guard waited before it paused this session; null unless it did and the
	 *  session is still paused. */
	let idlePaused = $state<number | null>(null);
	/** The amount and the hours this session has passed, once each, until dismissed. */
	let costPassed = $state<number | null>(null);
	let durationPassed = $state<number | null>(null);
	// Said once a session: dismissing does not bring it back at the next check.
	let costSaid = false;
	let durationSaid = false;

	function reset() {
		idlePaused = null;
		costPassed = null;
		durationPassed = null;
		costSaid = false;
		durationSaid = false;
	}

	function check() {
		if (!running.current) return;
		if (!billedStreamOpen(get(originStates), get(trackEngines), setup.current.provider)) return;
		const now = clock.now;
		if (
			!paused.current &&
			idlePauseDue(now, lastSignOfLife(get(activityTimes), get(runningSince)), minutes.current)
		) {
			idlePaused = minutes.current;
			// A pause that did not happen (refused, or a stop under way) says nothing.
			void pause().then(() => {
				if (!get(pauseRequested)) idlePaused = null;
			});
		}
		if (!costSaid && costWarningDue(cost(), threshold.current)) {
			costSaid = true;
			costPassed = threshold.current;
		}
		if (!durationSaid && durationWarningDue(clock.elapsedMs, hours.current)) {
			durationSaid = true;
			durationPassed = hours.current;
		}
	}

	const tick = $derived(Math.floor(clock.now / CHECK_EVERY_MS));
	$effect(() => {
		void tick;
		untrack(check);
	});
	// A new session starts with nothing said.
	$effect(() => {
		if (!running.current) untrack(reset);
	});
	// Resumed, by the notice's button, the toolbar or the shortcut: the reason for the pause
	// has gone with it.
	$effect(() => {
		if (!paused.current) idlePaused = null;
	});

	return {
		get idlePaused() {
			return idlePaused;
		},
		get costPassed() {
			return costPassed;
		},
		get durationPassed() {
			return durationPassed;
		},
		/** Put a notice away. The idle pause's goes on Resume by itself; this only hides it. */
		dismiss(notice: GuardNotice) {
			if (notice === 'idle') idlePaused = null;
			if (notice === 'cost') costPassed = null;
			if (notice === 'duration') durationPassed = null;
		}
	};
}
export type SpendingGuard = ReturnType<typeof createSpendingGuard>;
