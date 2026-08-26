// The transcript as an explicit document with a saved/unsaved state, plus the on-disk
// recovery snapshot format. Pure logic — no Svelte, no Tauri — so a long session, a
// duplicate save, a discard and a malformed recovery file can all be tested directly.
//
// Issue #25: the log used to be capped at 1,000 lines and silently truncated, and nothing
// on screen distinguished a saved transcript from one that only existed in memory.

import type { Origin, TranscriptLine } from './types';

/** Id meaning "nothing has been saved yet". Line ids start at 1. */
export const NOTHING_SAVED = 0;

/**
 * When the in-memory log gets this long, the operator is told to save.
 *
 * It is a warning, not a cap: nothing is ever dropped. At roughly ten finalized lines per
 * minute of speech this is around six hours of continuous captioning, so reaching it means
 * an unusually long session rather than a runaway.
 */
export const TRANSCRIPT_WARN_LINES = 4000;

/** Highest line id in a newest-first log; `NOTHING_SAVED` when the log is empty. */
export function newestLineId(newestFirst: TranscriptLine[]): number {
	return newestFirst.length ? newestFirst[0].id : NOTHING_SAVED;
}

/**
 * Whether the document holds finalized text that has not reached disk.
 *
 * Compared by line id rather than by a boolean flag so that saving twice with nothing new
 * in between stays saved, while one new line after a save makes the document unsaved again.
 * Ids are monotonic for the lifetime of the app, so a `Clear` followed by a fresh run cannot
 * make stale text look saved.
 */
export function isDirty(newestFirst: TranscriptLine[], savedLineId: number): boolean {
	return newestLineId(newestFirst) > savedLineId;
}

// ---- Recovery snapshot ------------------------------------------------------
// Written to the app's own local data directory when the operator opts in, so a crash or a
// power cut does not take the session with it. It holds finalized caption lines and nothing
// else: no API key, no audio, no device name, no provider identity.

export const RECOVERY_VERSION = 1;

export interface RecoverySnapshot {
	version: number;
	/** ISO-8601 UTC, for the "recovered from …" line in the restore offer. */
	savedAt: string;
	/** Same order as the store: newest first. */
	lines: TranscriptLine[];
}

const ORIGINS: readonly Origin[] = ['microphone', 'system'];

/** Serialize the log for the recovery file. Only the four caption fields are written. */
export function encodeRecovery(newestFirst: TranscriptLine[], savedAt: Date): string {
	const snapshot: RecoverySnapshot = {
		version: RECOVERY_VERSION,
		savedAt: savedAt.toISOString(),
		lines: newestFirst.map((line) => ({
			id: line.id,
			text: line.text,
			sourceText: line.sourceText,
			origin: line.origin
		}))
	};
	return JSON.stringify(snapshot);
}

function isLine(value: unknown): value is TranscriptLine {
	if (typeof value !== 'object' || value === null) return false;
	const line = value as Record<string, unknown>;
	return (
		typeof line.id === 'number' &&
		Number.isFinite(line.id) &&
		typeof line.text === 'string' &&
		typeof line.sourceText === 'string' &&
		typeof line.origin === 'string' &&
		(ORIGINS as readonly string[]).includes(line.origin)
	);
}

/**
 * Read a recovery file back. It is untrusted input — it survived a crash, it may have been
 * truncated mid-write, and it sits in a directory the user can edit — so anything that is
 * not a complete, current-version snapshot with at least one usable line yields null and the
 * app carries on as if there were nothing to recover.
 */
export function decodeRecovery(raw: string): RecoverySnapshot | null {
	let parsed: unknown;
	try {
		parsed = JSON.parse(raw);
	} catch {
		return null;
	}
	if (typeof parsed !== 'object' || parsed === null) return null;
	const snapshot = parsed as Record<string, unknown>;
	if (snapshot.version !== RECOVERY_VERSION) return null;
	if (typeof snapshot.savedAt !== 'string' || Number.isNaN(Date.parse(snapshot.savedAt))) {
		return null;
	}
	if (!Array.isArray(snapshot.lines)) return null;
	const lines = snapshot.lines.filter(isLine);
	if (!lines.length) return null;
	return { version: RECOVERY_VERSION, savedAt: snapshot.savedAt, lines };
}

// ---- Quit ------------------------------------------------------------------

/** What the operator chose in the unsaved-transcript prompt. */
export type CloseChoice = 'save' | 'discard' | 'cancel';

/**
 * Whether the Rust core should intercept a window close.
 *
 * A running session counts even with an empty transcript: closing mid-session has to stop
 * capture and let the provider flush its last turn before the document can be finalized,
 * which cannot happen if the window has already gone. When neither holds, close is left
 * alone entirely, so an unresponsive front-end can never make the window unclosable.
 */
export function shouldGuardClose(dirty: boolean, running: boolean): boolean {
	return dirty || running;
}
