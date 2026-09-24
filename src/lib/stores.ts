import { loadLanguageFavourites, LANGUAGE_FAVOURITES_KEY } from './languages';
import { HOLD_KEY, PACE_KEY, loadHoldSeconds, loadPace, type CaptionPace } from './reading';
import { sessionHistory } from './history';
import { CLEAN_SPEECH_KEY, loadCleanSpeech, loadFillerWords, saveFillerWords } from './cleanSpeech';
import { CAPTION_LAYOUT_KEY, loadCaptionLayout } from './captionLayout';
import type { CaptionLayout } from './captionLayout';
// Svelte stores shared across the operator window. The overlay window keeps its own
// minimal state (see routes/overlay/+page.svelte) so it stays lightweight.

import type { AppError } from './errors';
import { writable, derived, get } from 'svelte/store';
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
	loadOverlayFont,
	loadOverlayWidth,
	loadStartOptions,
	OVERLAY_FONT_KEY,
	OVERLAY_WIDTH_KEY,
	OVERLAY_PLACED_KEY,
	RECOVERY_ENABLED_KEY,
	SESSION_OPTIONS_KEY,
	SHOW_ORIGINAL_KEY,
	TRAY_HIDE_EXPLAINED_KEY
} from './types';
import {
	CAPTION_SCRIM_KEY,
	CAPTION_SCRIM_OPACITY_KEY,
	CAPTION_TEXT_KEY,
	captionContrast,
	loadCaptionPalette
} from './captionColour';
import type { CaptionPalette } from './captionColour';
import { CAPTION_FACE_KEY, loadCaptionFace } from './captionFont';
import type { CaptionFaceId } from './captionFont';
import { isDirty, newestLineId, NOTHING_SAVED } from './document';
import { persisted, persistedFlag, persistedWith, writeStored } from './persisted';
import { normalizeAppearance, type Appearance } from './appearance';

// ---- Session status --------------------------------------------------------
// Up to four backend tasks (two captures + two clients in "Both" mode) report status
// independently, so state is tracked per origin and aggregated for display: the worst
// state wins, and the session counts as active while any source still is.

export const originStates = writable<Partial<Record<Origin, SessionState>>>({});

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

/** Whether a source of the current run has reported an active state since `beginSession`.
 *  When a replacement starts, the drained session's Idle can arrive after `beginSession`;
 *  the core stops the old run before starting the new one, so that stale Idle always comes
 *  before this run's first active status — and must not be taken for this run ending. */
let runHadActiveSource = false;

/** Apply one status event from the Rust core. */
export function applyStatus(u: StatusUpdate) {
	if (u.origin) {
		const origin = u.origin;
		originStates.update((m) => ({ ...m, [origin]: u.state }));
		if (u.state === 'error' || u.state === 'idle') {
			const level = { source: origin, rms: 0, peak: 0 };
			(origin === 'microphone' ? micLevel : systemLevel).set(level);
			currentCaptions.update((m) => {
				const next = { ...m };
				delete next[origin];
				return next;
			});
			if (!get(isRunning)) {
				sessionStartedAt.set(null);
				// Every source of this run has ended by itself — a provider failure, or a
				// capture that died — with no Stop to close the record. Left open, it would be
				// closed by the next Start or quit and claim all the time in between.
				if (runHadActiveSource) {
					runHadActiveSource = false;
					void endTranscriptSession();
				}
			}
		} else {
			runHadActiveSource = true;
			// Starting a replacement first drains the old backend session, whose Idle
			// can arrive after beginSession. Its first active status starts the new clock.
			if (get(sessionStartedAt) === null) sessionStartedAt.set(Date.now());
		}
	} else if (u.state === 'idle') {
		// Whole-session stop: commit any in-flight caption so it can be saved, and
		// clear per-source state so the meters don't freeze at their last value.
		originStates.set({});
		activityTimes.set({});
		runHadActiveSource = false;
		flushTranscript();
		micLevel.set({ source: 'microphone', rms: 0, peak: 0 });
		systemLevel.set({ source: 'system', rms: 0, peak: 0 });
		currentCaptions.set({});
		sessionStartedAt.set(null);
	}
	if (u.message) {
		statusMessage.set(u.message);
	} else if (!u.origin && u.state === 'idle') {
		statusMessage.set('');
	}
}

export const activityTimes = writable<Partial<Record<Origin, { audio: number; caption: number }>>>(
	{}
);
// Notes arrive with every meter reading (20 Hz per source) and every caption, but they feed
// labels with a three-second threshold. One note per half-second is indistinguishable there,
// and keeps the store and the component reading it from updating dozens of times a second.
const ACTIVITY_RESOLUTION_MS = 500;

export function noteActivity(origin: Origin, kind: 'audio' | 'caption', now = Date.now()) {
	const last = get(activityTimes)[origin]?.[kind] ?? 0;
	if (last > 0 && now >= last && now - last < ACTIVITY_RESOLUTION_MS) return;
	activityTimes.update((value) => ({
		...value,
		[origin]: { audio: 0, caption: 0, ...value[origin], [kind]: now }
	}));
}
export const hasKey = writable<boolean>(false);

// Persisted to localStorage: the keyless built-in demo applies to a first run only, and a
// configured operator's setup survives a restart. A chosen application is stored without its
// process: a process id means nothing after a restart.
export const options = persistedWith<StartOptions>(loadStartOptions, (v) =>
	writeStored(
		SESSION_OPTIONS_KEY,
		JSON.stringify({
			...v,
			...(v.systemCapture?.kind === 'application'
				? { systemCapture: { kind: 'application', process: null } }
				: {})
		})
	)
);

// ---- Captions & transcript --------------------------------------------------

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
export const transcriptDirty = derived([transcript, savedLineId], ([lines, saved]) =>
	isDirty(lines, saved)
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
let transcriptTimeOffset = 0;

function commit(c: Caption) {
	if (!c.text.trim()) return;
	const line: TranscriptLine = {
		id: nextLineId++,
		text: c.text.trim(),
		sourceText: c.sourceText.trim(),
		origin: c.origin,
		// The committed caption's own interval: the turn's start, and the moment this text
		// was the last thing the provider had to say about it.
		startMs: c.startMs === undefined ? undefined : c.startMs + transcriptTimeOffset,
		endMs: c.endMs === undefined ? undefined : c.endMs + transcriptTimeOffset
	};
	transcript.update((list) => [line, ...list]);
	sessionHistory.append({ ...line, startMs: c.startMs, endMs: c.endMs });
}

export function pushCaption(c: Caption) {
	noteActivity(c.origin, 'caption');
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

/** End the run's document: commit in-flight lines, then close its history record. Safe to
 *  call more than once — a finished record keeps its first end time. */
export function endTranscriptSession(): Promise<void> {
	flushTranscript();
	return sessionHistory.finish();
}

/** Prepare the monitor for a new run without discarding already finalized transcript lines. */
export function beginSession(sessionOptions = get(options)) {
	void endTranscriptSession();
	runHadActiveSource = false;
	sessionHistory.begin(sessionOptions);
	// Retried/new sessions append to one document without resetting its cue timeline.
	transcriptTimeOffset = get(transcript).reduce((end, line) => Math.max(end, line.endMs ?? 0), 0);
	originStates.set({});
	micLevel.set({ source: 'microphone', rms: 0, peak: 0 });
	systemLevel.set({ source: 'system', rms: 0, peak: 0 });
	currentCaptions.set({});
	sessionStartedAt.set(Date.now());
}

/** Clear both finalized and in-flight transcript state, and the saved marker with them —
 *  an empty document is neither saved nor unsaved, and the next run's line ids continue
 *  upward, so a stale marker could otherwise make fresh text look already written. */
export function clearTranscript() {
	transcriptTimeOffset = 0;
	transcript.set([]);
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

export const overlayFontSize = persisted<number>(OVERLAY_FONT_KEY, loadOverlayFont);

/** Fit window, Compact or Stable reading. Shared with the overlay window through the same
 *  localStorage origin. */
export const overlayCaptionLayout = persisted<CaptionLayout>(CAPTION_LAYOUT_KEY, loadCaptionLayout);

/** How wide a caption line may run, in `ch`. Persisted like the font size. */
export const overlayCaptionWidth = persisted<number>(OVERLAY_WIDTH_KEY, loadOverlayWidth);

/** The caption typeface. Persisted and shared the same way; the id is validated on read, so
 *  a hand-edited or stale value falls back to the bundled default rather than to nothing. */
export const overlayCaptionFace = persisted<CaptionFaceId>(CAPTION_FACE_KEY, loadCaptionFace);

/** The caption ink and the scrim behind it, as one store: they are chosen together and judged
 *  together, and a contrast reading of half a palette would mean nothing. */
export const overlayPalette = persistedWith<CaptionPalette>(loadCaptionPalette, (p) => {
	writeStored(CAPTION_TEXT_KEY, p.text);
	writeStored(CAPTION_SCRIM_KEY, p.scrim);
	writeStored(CAPTION_SCRIM_OPACITY_KEY, String(p.scrimOpacity));
});

/** What the palette actually achieves on a projector, recomputed as it changes. Derived
 *  rather than stored: it is a fact about the palette, and a cached one could disagree. */
export const overlayContrast = derived(overlayPalette, ($p) => captionContrast($p));

// Whether the caption region has been positioned on the presentation display; persisted so
// the pre-flight check survives a restart.
export const overlayPlaced = persistedFlag(OVERLAY_PLACED_KEY);

// ---- Crash recovery ----------------------------------------------------------
// Off until the operator asks for it: writing captions to disk on a timer is exactly what
// the privacy policy promises the app does not do by default. Persisted so a room that
// wants the safety net does not have to re-enable it before every event.

export const recoveryEnabled = persistedFlag(RECOVERY_ENABLED_KEY);

// ---- Bilingual output ----------------------------------------------------------
// Translation keeps what was said before it was translated. Off by default, both of them:
// a bilingual document or overlay is a choice, not something an existing room should find
// changed after an update.

/** Write the original speech under each translation in a saved transcript. */
export const exportOriginal = persistedFlag('transcript.includeOriginal');

/** Show the original speech as a smaller line under each translated caption. */
export const overlayShowOriginal = persistedFlag(SHOW_ORIGINAL_KEY);

// ---- Window and tray ----------------------------------------------------------
// Off by default, so a fresh install keeps ordinary Windows semantics: the X closes the app.
// Staying alive after being closed is a thing an operator opts into, usually once, for a room
// where the window is in the way but the session must not stop.

export const closeToTray = persistedFlag(CLOSE_TO_TRAY_KEY);

/** Whether the operator has already been told that closing no longer quits. */
export const trayHideExplained = persistedFlag(TRAY_HIDE_EXPLAINED_KEY);

// ---- Audio levels ------------------------------------------------------------

export const micLevel = writable<AudioLevel>({ source: 'microphone', rms: 0, peak: 0 });
export const systemLevel = writable<AudioLevel>({ source: 'system', rms: 0, peak: 0 });

export const overlayCleanSpeech = persisted(CLEAN_SPEECH_KEY, loadCleanSpeech);
/** The words Hide filler words removes. Outside `appearance` on purpose: a curated list is
 *  not something a profile or Reset appearance should overwrite. It has its own reset. */
export const overlayFillerWords = persistedWith<readonly string[]>(
	loadFillerWords,
	saveFillerWords
);
export const overlayHoldSeconds = persisted(HOLD_KEY, loadHoldSeconds);
export const overlayPace = persisted<CaptionPace>(PACE_KEY, loadPace);

/** Operator preference, deliberately outside StartOptions and IPC. */
export const languageFavourites = persistedWith(
	() => loadLanguageFavourites(),
	(codes) => writeStored(LANGUAGE_FAVOURITES_KEY, JSON.stringify(codes))
);

// ---- Appearance as a whole ---------------------------------------------------------

/** The eight appearance stores as one value, for saving, comparing and pushing. */
export const appearance = derived(
	[
		overlayFontSize,
		overlayCaptionWidth,
		overlayCaptionLayout,
		overlayCaptionFace,
		overlayPalette,
		overlayCleanSpeech,
		overlayHoldSeconds,
		overlayPace
	],
	([fontSize, width, layout, face, palette, cleanSpeech, hold, pace]): Appearance => ({
		fontSize,
		width,
		layout,
		face,
		palette,
		cleanSpeech,
		hold,
		pace
	})
);

/** Set every appearance store from one value, normalized first so nothing out of range is
 *  stored or shown in the operator window's readouts. */
export function applyAppearance(value: Appearance) {
	const a = normalizeAppearance(value);
	overlayFontSize.set(a.fontSize);
	overlayCaptionWidth.set(a.width);
	overlayCaptionLayout.set(a.layout);
	overlayCaptionFace.set(a.face);
	overlayPalette.set(a.palette);
	overlayCleanSpeech.set(a.cleanSpeech);
	overlayHoldSeconds.set(a.hold);
	overlayPace.set(a.pace);
}
