import { get, writable } from 'svelte/store';
import { asStatus } from './errors';
import { applyStatus, beginSession, isRunning, statusMessage } from './stores';
import { api } from './tauri';
import type { StartOptions } from './types';

/** Own start/stop serialization separately from page rendering. A stop requested during
 * startup waits for that startup; it must not silently return while capture comes alive. */
export function createSessionController(
	port = {
		startSession: api.startSession,
		stopSession: api.stopSession
	}
) {
	const busy = writable(false);
	let operation: Promise<unknown> | null = null;
	let stopping: Promise<void> | null = null;

	return {
		busy,
		async start(options: StartOptions): Promise<boolean> {
			if (operation || stopping || get(isRunning)) return false;
			busy.set(true);
			statusMessage.set('');
			beginSession();
			const work = Promise.resolve().then(() => port.startSession(options));
			operation = work;
			try {
				await work;
				return true;
			} catch (error) {
				applyStatus({ state: 'idle' });
				statusMessage.set(asStatus(error));
				return false;
			} finally {
				operation = null;
				if (!stopping) busy.set(false);
			}
		},
		stop(): Promise<void> {
			if (stopping) return stopping;
			const startup = operation;
			busy.set(true);
			stopping = (async () => {
				await startup?.catch(() => {});
				try {
					await port.stopSession();
				} catch (error) {
					statusMessage.set(asStatus(error));
				} finally {
					stopping = null;
					busy.set(false);
				}
			})();
			return stopping;
		}
	};
}
