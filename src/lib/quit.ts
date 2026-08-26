// Closing the operator window, in the order the guarantees have to happen (issue #25):
// stop capture, let the providers hand over their last turn, finalize the document, and only
// then decide whether the operator has to be asked anything.
//
// It lives here rather than in the page so the sequence can be tested directly — a graceful
// quit is exactly the path nobody exercises by hand until the one event where it matters.

import { get } from 'svelte/store';

import type { CloseChoice } from './document';
import { flushTranscript, isRunning, transcriptDirty } from './stores';
import { saveTranscriptDocument } from './saveDocument';
import { api } from './tauri';

/** A provider that never finishes flushing must not hold the window hostage. `stop_session`
 *  bounds its own drain at five seconds; this bounds the wait on the whole call. */
export const QUIT_DRAIN_TIMEOUT_MS = 8000;

export interface CloseOutcome {
	/** A session was running and has been stopped and drained, so the prompt can explain
	 *  where the last few lines came from. */
	endedSession: boolean;
	/** Unsaved lines remain: the operator has to answer before the app can quit. When false,
	 *  the quit has already been confirmed and the window is on its way out. */
	prompt: boolean;
}

function withTimeout(work: Promise<unknown>, ms: number): Promise<unknown> {
	let timer: ReturnType<typeof setTimeout>;
	return Promise.race([
		// The caller reports its own failures; here a rejected stop must not turn into an
		// unhandled rejection once the race has already been settled by the timeout.
		work.catch(() => {}),
		new Promise((resolve) => {
			timer = setTimeout(resolve, ms);
		})
	]).finally(() => clearTimeout(timer));
}

/**
 * Everything that must happen before a close can be allowed or questioned.
 *
 * `stopSession` is injected because the page owns the busy flag and the status line that a
 * stop touches; this only needs to know when it has finished.
 */
export async function prepareClose(
	stopSession: () => Promise<void>,
	timeoutMs = QUIT_DRAIN_TIMEOUT_MS
): Promise<CloseOutcome> {
	// First, before the drain: the core releases the window if nothing claims the close within
	// a few seconds, and stopping a session takes longer than that.
	await api.ackClose().catch(() => {});

	let endedSession = false;
	if (get(isRunning)) {
		endedSession = true;
		await withTimeout(stopSession(), timeoutMs);
	}
	// A turn that never got an explicit turn-complete is still the operator's text.
	flushTranscript();

	if (get(transcriptDirty)) return { endedSession, prompt: true };

	await api.confirmClose();
	return { endedSession, prompt: false };
}

/**
 * Act on the operator's answer and quit. `cancel` is handled by the caller — it is the one
 * answer that changes nothing.
 *
 * A failed save throws and the app stays open: quitting on a write that did not land would
 * lose precisely what the operator just asked to keep.
 */
export async function resolveClose(choice: Exclude<CloseChoice, 'cancel'>): Promise<void> {
	if (choice === 'save') {
		await saveTranscriptDocument('markdown');
	} else {
		// The spool only ever held the text now being thrown away.
		await api.clearRecovery().catch(() => {});
	}
	await api.confirmClose();
}
