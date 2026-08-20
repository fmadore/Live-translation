// Svelte stores shared across the operator window. The overlay window keeps its own
// minimal state (see routes/overlay/+page.svelte) so it stays lightweight.

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
	loadOverlayFont,
	loadOverlayPlaced,
	loadStartOptions,
	OVERLAY_FONT_KEY,
	OVERLAY_PLACED_KEY,
	SESSION_OPTIONS_KEY
} from './types';

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

export const statusMessage = writable<string>('');

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

// Persisted to localStorage: the keyless on-device default applies to a first run only, and a
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

// Rolling transcript log (most recent finalized lines first).
export const transcript = writable<TranscriptLine[]>([]);

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
	transcript.update((list) => [line, ...list].slice(0, 1000));
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

/** Clear both finalized and in-flight transcript state. */
export function clearTranscript() {
	transcript.set([]);
	latestCaption.set(null);
	currentCaptions.set({});
	for (const origin of Object.keys(pending) as Origin[]) delete pending[origin];
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

// ---- Audio levels ------------------------------------------------------------

export const micLevel = writable<AudioLevel>({ source: 'microphone', rms: 0, peak: 0 });
export const systemLevel = writable<AudioLevel>({ source: 'system', rms: 0, peak: 0 });
