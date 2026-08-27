// Svelte stores shared across the operator window. The overlay window keeps its own
// minimal state (see routes/overlay/+page.svelte) so it stays lightweight.

import type { AppError } from './errors';
import { writable, derived } from 'svelte/store';
import type {
	AudioLevel,
	Caption,
	Origin,
	SessionState,
	StartOptions,
	StatusUpdate,
	TranscriptLine
} from './types';
import {
	CLOSE_TO_TRAY_KEY,
	loadCloseToTray,
	loadOverlayFont,
	loadOverlayPlaced,
	loadRecoveryEnabled,
	loadStartOptions,
	loadTrayHideExplained,
	OVERLAY_FONT_KEY,
	OVERLAY_PLACED_KEY,
	RECOVERY_ENABLED_KEY,
	SESSION_OPTIONS_KEY,
	TRAY_HIDE_EXPLAINED_KEY
} from './types';
import { isDirty, newestLineId, NOTHING_SAVED } from './document';

// ---- Session status --------------------------------------------------------
// Up to four backend tasks (two captures + two clients in "Both" mode) report status
// independently, so state is tracked per origin and aggregated for display: the worst
// state wins, and the session counts as active while any source still is.

const originStates = writable<Partial<Record<Origin, SessionState>>>({});

const DISPLAY_PRIORITY: SessionState[] = ['error', 'reconnecting', 'connecting', 'running'];

export const sessionState = derived(originStates, (m): SessionState => {
	const states = Object.values(m);
	return DISPLAY_PRIORITY.find((s) => states.includes(s)) ?? 'idle';
});

export const isRunning = derived(originStates, (m) =>
	Object.values(m).some((s) => s === 'running' || s === 'reconnecting' || s === 'connecting')
);

// Either plain text or the core's structured error. Structured, because the sentence for an
// id belongs to the interface language and is chosen where it is rendered — see
// `describeError`.
export const statusMessage = writable<string | AppError>('');

/** Wall-clock start of the current run, or null when idle. Drives the elapsed timer and the
 *  running cost estimate. */
export const sessionStartedAt = writable<number | null>(null);

/** Apply one status event from the Rust core. */
export function applyStatus(u: StatusUpdate) {
	if (u.origin) {
		const origin = u.origin;
		originStates.update((m) => ({ ...m, [origin]: u.state }));
	} else if (u.state === 'idle') {
		// Whole-session stop: commit any in-flight caption so it can be saved, and
		// clear per-source state so the meters don't freeze at their last value.
		originStates.set({});
		flushTranscript();
		micLevel.set({ source: 'microphone', rms: 0, peak: 0 });
		systemLevel.set({ source: 'system', rms: 0, peak: 0 });
		latestCaption.set(null);
		currentCaptions.set({});
		sessionStartedAt.set(null);
	}
	if (u.message) {
		statusMessage.set(u.message);
	} else if (!u.origin && u.state === 'idle') {
		statusMessage.set('');
	}
}

export const hasKey = writable<boolean>(false);

// Persisted to localStorage: the keyless built-in demo applies to a first run only, and a
// configured operator's setup survives a restart.
export const options = writable<StartOptions>(loadStartOptions());

options.subscribe((v) => {
	if (typeof localStorage !== 'undefined') {
		localStorage.setItem(SESSION_OPTIONS_KEY, JSON.stringify(v));
	}
});

// ---- Captions & transcript --------------------------------------------------

// Latest caption (any origin), for the operator monitor.
export const latestCaption = writable<Caption | null>(null);

// The turn currently on screen for each origin, so the operator can show both speakers at
// once. Key insertion order is kept in least-recently-updated order, which is the order the
// stage renders the blocks in — newest at the bottom.
export const currentCaptions = writable<Partial<Record<Origin, Caption>>>({});

// The transcript log, most recent finalized line first.
//
// Deliberately unbounded. It used to be capped at 1,000 lines, which silently dropped the
// beginning of any long event — exactly the sessions worth keeping (issue #25). A finalized
// line is a sentence or two of text, so even a day-long session is a few megabytes; the
// operator is warned to save well before then (`TRANSCRIPT_WARN_LINES`) rather than losing
// anything behind their back.
export const transcript = writable<TranscriptLine[]>([]);

// ---- Saved / unsaved document state -----------------------------------------

/** Highest line id written to disk, so a second save with nothing new stays saved. */
export const savedLineId = writable<number>(NOTHING_SAVED);

/** Path of the last successful save, shown next to the saved badge. */
export const savedPath = writable<string>('');

/** True while the log holds finalized text that has not reached disk. */
export const transcriptDirty = derived(
	[transcript, savedLineId],
	([lines, saved]) => isDirty(lines, saved)
);

/** Record a successful write: everything logged up to now is on disk at `path`. */
export function markTranscriptSaved(lines: TranscriptLine[], path: string) {
	savedLineId.set(newestLineId(lines));
	savedPath.set(path);
}

// Track the in-flight turn *per origin* so a transcript line is logged even when a stream
// never emits an explicit turn-complete, and so mic and system turns (whose ids are
// independent counters) never clobber each other in "Both" mode.
const pending: Partial<Record<Origin, Caption>> = {};

let nextLineId = 1;

function commit(c: Caption) {
	if (!c.text.trim()) return;
	const line: TranscriptLine = {
		id: nextLineId++,
		text: c.text.trim(),
		sourceText: c.sourceText.trim(),
		origin: c.origin
	};
	transcript.update((list) => [line, ...list]);
}

export function pushCaption(c: Caption) {
	latestCaption.set(c);
	// Re-insert this origin last so the object's key order tracks recency.
	currentCaptions.update((m) => {
		const next: Partial<Record<Origin, Caption>> = {};
		for (const o of Object.keys(m) as Origin[]) if (o !== c.origin) next[o] = m[o];
		next[c.origin] = c;
		return next;
	});
	const prev = pending[c.origin];
	// A new turn id means this origin's previous turn is done, even without an explicit
	// turn-complete.
	if (prev && prev.turnId !== c.turnId) {
		commit(prev);
		delete pending[c.origin];
	}
	if (c.final) {
		commit(c);
		delete pending[c.origin];
	} else {
		pending[c.origin] = c;
	}
}

/** Commit all in-flight lines (call when the session ends) so they aren't lost. */
export function flushTranscript() {
	for (const origin of Object.keys(pending) as Origin[]) {
		const c = pending[origin];
		if (c) commit(c);
		delete pending[origin];
	}
}

/** Prepare the monitor for a new run without discarding already finalized transcript lines. */
export function beginSession() {
	flushTranscript();
	latestCaption.set(null);
	currentCaptions.set({});
	sessionStartedAt.set(Date.now());
}

/** Clear both finalized and in-flight transcript state, and the saved marker with them —
 *  an empty document is neither saved nor unsaved, and the next run's line ids continue
 *  upward, so a stale marker could otherwise make fresh text look already written. */
export function clearTranscript() {
	transcript.set([]);
	latestCaption.set(null);
	currentCaptions.set({});
	savedLineId.set(NOTHING_SAVED);
	savedPath.set('');
	for (const origin of Object.keys(pending) as Origin[]) delete pending[origin];
}

/** Replace the log with a recovered snapshot. The restored lines are unsaved by definition —
 *  the file they came from is a crash spool, not the operator's transcript — and ids continue
 *  above the snapshot so a later run cannot collide with them. */
export function restoreTranscript(lines: TranscriptLine[]) {
	transcript.set(lines);
	savedLineId.set(NOTHING_SAVED);
	savedPath.set('');
	nextLineId = Math.max(nextLineId, newestLineId(lines) + 1);
}

// ---- Overlay font size ------------------------------------------------------
// Persisted to localStorage so both windows share the same default.

export const overlayFontSize = writable<number>(loadOverlayFont());

overlayFontSize.subscribe((v) => {
	if (typeof localStorage !== 'undefined') localStorage.setItem(OVERLAY_FONT_KEY, String(v));
});

// Whether the caption region has been positioned on the presentation display; persisted so
// the pre-flight check survives a restart.
export const overlayPlaced = writable<boolean>(loadOverlayPlaced());

overlayPlaced.subscribe((v) => {
	if (typeof localStorage !== 'undefined') localStorage.setItem(OVERLAY_PLACED_KEY, String(v));
});

// ---- Crash recovery ----------------------------------------------------------
// Off until the operator asks for it: writing captions to disk on a timer is exactly what
// the privacy policy promises the app does not do by default. Persisted so a room that
// wants the safety net does not have to re-enable it before every event.

export const recoveryEnabled = writable<boolean>(loadRecoveryEnabled());

recoveryEnabled.subscribe((v) => {
	if (typeof localStorage !== 'undefined') localStorage.setItem(RECOVERY_ENABLED_KEY, String(v));
});

// ---- Window and tray ----------------------------------------------------------
// Off by default, so a fresh install keeps ordinary Windows semantics: the X closes the app.
// Staying alive after being closed is a thing an operator opts into, usually once, for a room
// where the window is in the way but the session must not stop.

export const closeToTray = writable<boolean>(loadCloseToTray());

closeToTray.subscribe((v) => {
	if (typeof localStorage !== 'undefined') localStorage.setItem(CLOSE_TO_TRAY_KEY, String(v));
});

/** Whether the operator has already been told that closing no longer quits. */
export const trayHideExplained = writable<boolean>(loadTrayHideExplained());

trayHideExplained.subscribe((v) => {
	if (typeof localStorage !== 'undefined') {
		localStorage.setItem(TRAY_HIDE_EXPLAINED_KEY, String(v));
	}
});

// ---- Audio levels ------------------------------------------------------------

export const micLevel = writable<AudioLevel>({ source: 'microphone', rms: 0, peak: 0 });
export const systemLevel = writable<AudioLevel>({ source: 'system', rms: 0, peak: 0 });
