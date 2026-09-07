import { encodeRecovery, newestLineId } from './document';
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

/** Scheduling is separate from the queue so unmounting/re-enabling cannot create a
 * second writer. The queue also serves Save, Clear, Disable, Restore and Quit. */
export function startRecoverySpool(
	readUnsaved: () => TranscriptLine[] | null,
	onError: (error: unknown) => void,
	coordinator = recovery
) {
	let writing = false;
	let stopped = false;
	const timer = setInterval(() => {
		if (writing || stopped) return;
		const lines = readUnsaved();
		if (!lines?.length) return;
		writing = true;
		void coordinator
			.write(lines)
			.catch((error) => {
				if (stopped) return;
				stopped = true;
				onError(error);
			})
			.finally(() => {
				writing = false;
			});
	}, RECOVERY_INTERVAL_MS);
	return () => {
		stopped = true;
		clearInterval(timer);
	};
}
