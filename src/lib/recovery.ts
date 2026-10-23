import { encodeRecovery, newestLineId } from './document';
import { asStatus, type AppError } from './errors';
import { writable } from 'svelte/store';
import { api } from './tauri';
import type { StoredRecovery, TranscriptLine } from './types';

interface RecoveryPort {
	writeRecovery(contents: string): Promise<string>;
	clearRecovery(): Promise<void>;
	readRecovery(): Promise<StoredRecovery | null>;
}

/** One queue for every recovery reader, writer and deletion in the operator window.
 * Clear invalidates queued snapshots immediately and waits for an active write before
 * deleting it. Errors reach the caller without poisoning later queue operations. */
export function createRecoveryCoordinator(port: RecoveryPort) {
	let queue: Promise<unknown> = Promise.resolve();
	let generation = 0;
	let spooledId: number | null = null;

	function enqueue<T>(work: () => Promise<T>): Promise<T> {
		const result = queue.then(work);
		queue = result.catch(() => {});
		return result;
	}

	return {
		write(lines: TranscriptLine[], now = new Date()): Promise<void> {
			const revision = generation;
			const newest = newestLineId(lines);
			return enqueue(async () => {
				if (revision !== generation || !lines.length || newest === spooledId) return;
				await port.writeRecovery(encodeRecovery(lines, now));
				if (revision === generation) spooledId = newest;
			});
		},
		clear(): Promise<void> {
			generation++;
			spooledId = null;
			return enqueue(() => port.clearRecovery());
		},
		read(): Promise<StoredRecovery | null> {
			return enqueue(() => port.readRecovery());
		}
	};
}

export const recovery = createRecoveryCoordinator({
	writeRecovery: (contents) => api.writeRecovery(contents),
	clearRecovery: () => api.clearRecovery(),
	readRecovery: () => api.readRecovery()
});

export const RECOVERY_INTERVAL_MS = 8000;

/** A failed spool write, in the shape the status line holds. The core names its own failure;
 *  anything else that stopped the write is still reported as the spool's, with what was thrown
 *  as the detail, so the operator learns what stopped rather than only that something did. */
export function spoolFailure(error: unknown): string | AppError {
	const status = asStatus(error);
	return typeof status === 'string' ? { id: 'error.recoveryWrite', detail: status } : status;
}

/** The longest the spool waits between attempts while its writes keep failing. */
export const RECOVERY_MAX_BACKOFF_MS = 64_000;

/** Why the spool's last write failed, while it keeps failing; empty while it works, and while
 *  it is off. Held for as long as the failure lasts, so the transcript can say so beside its
 *  save badge rather than only on the status line, which the next status overwrites. */
export const recoveryFailure = writable<string | AppError>('');

/** Scheduling is separate from the queue so unmounting/re-enabling cannot create a
 * second writer. The queue also serves Save, Clear, Disable, Restore and Quit.
 *
 * A failed write does not stop the spool: a full disk gets space back, a locked file is let
 * go. It tries again after twice the wait each time, up to `RECOVERY_MAX_BACKOFF_MS`, and
 * reports once per run of failures rather than on every attempt. A write that succeeds — or
 * nothing left unsaved to protect — ends the run. */
export function startRecoverySpool(
	readUnsaved: () => TranscriptLine[] | null,
	onError: (error: unknown) => void,
	coordinator = recovery
) {
	let writing = false;
	let stopped = false;
	let failures = 0;
	let retryAt = 0;
	const recovered = () => {
		failures = 0;
		retryAt = 0;
		recoveryFailure.set('');
	};
	const timer = setInterval(() => {
		if (writing || stopped || Date.now() < retryAt) return;
		const lines = readUnsaved();
		if (!lines?.length) {
			if (failures) recovered();
			return;
		}
		writing = true;
		void coordinator
			.write(lines)
			.then(() => {
				if (!stopped && failures) recovered();
			})
			.catch((error) => {
				if (stopped) return;
				failures++;
				retryAt =
					Date.now() + Math.min(RECOVERY_INTERVAL_MS * 2 ** failures, RECOVERY_MAX_BACKOFF_MS);
				recoveryFailure.set(spoolFailure(error));
				if (failures === 1) onError(error);
			})
			.finally(() => {
				writing = false;
			});
	}, RECOVERY_INTERVAL_MS);
	return () => {
		stopped = true;
		clearInterval(timer);
		// Switched off, the spool is not failing: there is nothing it is meant to be writing.
		recoveryFailure.set('');
	};
}
