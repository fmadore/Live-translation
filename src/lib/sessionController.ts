import { get, writable } from 'svelte/store';
import { asStatus } from './errors';
import {
	applyStatus,
	beginSession,
	endTranscriptSession,
	heldRunEnd,
	isRunning,
	pauseRequested,
	sourcesActive,
	statusMessage
} from './stores';
import { api } from './tauri';
import { secondCaptionLanguageOf, type SourceSwitch, type StartOptions } from './types';
import { captionLanguageError } from './languages';
import { spokenLanguageFor } from './providers';
import { locale, t } from './i18n';

/** Own start/stop serialization separately from page rendering. A stop requested during
 * startup waits for that startup; it must not silently return while capture comes alive. */
export function createSessionController(
	port: {
		startSession: (options: StartOptions) => Promise<void>;
		stopSession: () => Promise<void>;
		pauseSession: (paused: boolean) => Promise<void>;
		switchSourceEngine?: (request: SourceSwitch) => Promise<void>;
	} = {
		startSession: api.startSession,
		stopSession: api.stopSession,
		pauseSession: api.pauseSession,
		switchSourceEngine: api.switchSourceEngine
	}
) {
	const busy = writable(false);
	let startup: Promise<void> | null = null;
	let switching: Promise<void> | null = null;
	let stopping: Promise<void> | null = null;

	return {
		busy,
		async start(options: StartOptions): Promise<boolean> {
			if (startup || switching || stopping || get(isRunning)) return false;
			if (captionLanguageError(options, get(t), get(locale))) {
				// Worded where it is shown, from the setup that was refused, so a change of
				// interface language re-words it there.
				statusMessage.set((m, tag) => captionLanguageError(options, m, tag));
				return false;
			}
			const second = secondCaptionLanguageOf(options);
			busy.set(true);
			statusMessage.set('');
			// A second language chosen for translation stays in the saved setup when the mode
			// changes, so switching back restores it; only a run that can use it sends it.
			// Likewise the spoken language: only an engine that takes it gets it, and Gemini
			// Transcribe only one it knows, as its picker shows (#249).
			const run: StartOptions = {
				...options,
				secondTargetLanguage: second ?? null,
				spokenLanguage: spokenLanguageFor(options.provider, options.spokenLanguage)
			};
			beginSession(run);
			const work = Promise.resolve().then(() => port.startSession(run));
			startup = work;
			try {
				await work;
				return true;
			} catch (error) {
				// The core sends no Idle for a start that failed; the window ends the run itself.
				applyStatus({ state: 'idle' }, { local: true });
				statusMessage.set(asStatus(error));
				return false;
			} finally {
				startup = null;
				if (!stopping) busy.set(false);
			}
		},
		/** Pause or resume the running session. Ignored while a start or stop is under way:
		 *  the session it would address is the one being replaced. */
		async pause(paused: boolean): Promise<void> {
			// A held run has no source left to pause (#248).
			if (startup || switching || stopping || !get(sourcesActive)) return;
			try {
				await port.pauseSession(paused);
				pauseRequested.set(paused);
			} catch (error) {
				statusMessage.set(asStatus(error));
			}
		},
		/** Continue one source. Stop can interrupt the load and takes over reporting its outcome. */
		async switchSource(request: SourceSwitch): Promise<boolean> {
			if (startup || switching || stopping) return false;
			const work = Promise.resolve().then(() =>
				(port.switchSourceEngine ?? api.switchSourceEngine)(request)
			);
			switching = work;
			try {
				await work;
				return !stopping;
			} catch (error) {
				if (stopping) return false;
				throw error;
			} finally {
				switching = null;
			}
		},
		stop(): Promise<void> {
			if (stopping) return stopping;
			const pendingSwitch = switching;
			busy.set(true);
			stopping = (async () => {
				await startup?.catch(() => {});
				// A held run ended when its last source did, not when it is stopped.
				const heldEnd = heldRunEnd();
				try {
					await port.stopSession();
					await endTranscriptSession(heldEnd);
				} catch (error) {
					statusMessage.set(asStatus(error));
				} finally {
					await pendingSwitch?.catch(() => {});
					stopping = null;
					busy.set(false);
				}
			})();
			return stopping;
		}
	};
}
