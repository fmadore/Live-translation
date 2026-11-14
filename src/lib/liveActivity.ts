import type { SessionState } from './types';
export type Activity =
	| 'idle'
	| 'error'
	| 'connecting'
	| 'reconnecting'
	| 'paused'
	| 'captions'
	| 'audio'
	| 'listening'
	| 'silenced'
	| 'stale';
/** What one source is doing, for its activity row. `runningSince` is when it last came to be
 *  running, not when the session began: audio keeps arriving while a source is paused or
 *  reconnecting, and that time is no evidence that captions have gone missing. */
export function activity(
	state: SessionState,
	now: number,
	runningSince: number,
	lastAudio: number,
	lastCaption: number,
	lastSilenced = 0
): Activity {
	if (state !== 'running') return state;
	if (now - lastCaption < 3000 && lastCaption > 0) return 'captions';
	if (lastAudio > 0 && now - lastAudio < 3000)
		return now - Math.max(runningSince, lastCaption) >= 15000 ? 'stale' : 'audio';
	// Not a quiet room: something plays, and the system hands over nothing but zeros.
	if (lastSilenced > 0 && now - lastSilenced < 3000) return 'silenced';
	return 'listening';
}
