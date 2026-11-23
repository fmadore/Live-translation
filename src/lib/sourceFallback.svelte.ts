// A cloud source that ended mid-session, and the operator's way on: continue it with offline
// Whisper in the same run (#248, docs/decisions/2026-11-23-fallback-engine.md). Modelled on
// `deviceFailure.svelte.ts`, but nothing is stopped: the core rebuilds that one source in place,
// so the clock, transcript, markers and history record carry on, and the other source with them.

import { get } from 'svelte/store';
import { asStatus } from './errors';
import { t } from './i18n';
import { isLocalWhisper } from './providers';
import {
	addMarker,
	endHeldRun,
	endsForFallback,
	heldSince,
	sessionStartedAt,
	sourcesActive,
	statusMessage,
	trackEngines
} from './stores';
import { ORIGINS, laneCount, whisperModelTranslates } from './types';
import type {
	Origin,
	Provider,
	SourceSwitch,
	StartOptions,
	StatusUpdate,
	TargetLanguage,
	WhisperModelId,
	WhisperModelInfo
} from './types';

/** How long a source reconnects before the banner offers Whisper beside the reconnection,
 *  which carries on until the operator chooses (decision 3). */
export const RECONNECT_OFFER_MS = 30_000;

/** Most accurate first: what "the best installed model" means when the setup's is not there. */
const BEST_FIRST: readonly WhisperModelId[] = ['turbo', 'small', 'base', 'tiny'];

/** Why the captions will not stay in the caption language, said on the button (decision 4). */
type FallbackNote =
	| { kind: 'keepsEnglish' }
	| { kind: 'turboCannotTranslate' }
	| { kind: 'cannotTranslate'; language: TargetLanguage };

/** What the banner offers: offline Whisper, with which task and model, and what it changes. */
export interface FallbackOffer {
	to: Extract<Provider, 'whisper' | 'whisper-translate'>;
	model: WhisperModelId;
	gpu: boolean;
	note: FallbackNote | null;
}

/** What the window already knows that decides the offer. */
export interface FallbackContext {
	setup: StartOptions;
	models: readonly WhisperModelInfo[];
	/** The processor runs the Whisper engine. */
	cpuSupported: boolean;
	/** A rehearsal or the demonstration is running: neither is offered a fallback. */
	rehearsal: boolean;
}

/**
 * Whether offline Whisper can continue this run, and how: the setup's model if it is
 * installed (and translates, when that is wanted), else the best installed model that fits.
 * Not for a rehearsal, the demonstration, a run already on Whisper or in two caption
 * languages, a processor that cannot run it, or with no model installed.
 */
export function whisperOffer(ctx: FallbackContext): FallbackOffer | null {
	const { setup } = ctx;
	if (!ctx.cpuSupported || ctx.rehearsal) return null;
	if (setup.provider === 'ondevice' || isLocalWhisper(setup.provider)) return null;
	if (laneCount(setup) === 2) return null;
	const installed = ctx.models.filter((m) => m.installed && !m.downloading).map((m) => m.id);
	const pick = (fits: (id: WhisperModelId) => boolean) =>
		setup.whisperModel && installed.includes(setup.whisperModel) && fits(setup.whisperModel)
			? setup.whisperModel
			: BEST_FIRST.find((id) => installed.includes(id) && fits(id));
	const any = pick(() => true);
	if (!any) return null;
	const gpu = setup.whisperGpu ?? true;
	if (setup.mode !== 'translate') return { to: 'whisper', model: any, gpu, note: null };
	if (setup.targetLanguage === 'en') {
		const translating = pick(whisperModelTranslates);
		return translating
			? { to: 'whisper-translate', model: translating, gpu, note: { kind: 'keepsEnglish' } }
			: { to: 'whisper', model: any, gpu, note: { kind: 'turboCannotTranslate' } };
	}
	return {
		to: 'whisper',
		model: any,
		gpu,
		note: { kind: 'cannotTranslate', language: setup.targetLanguage }
	};
}

export interface SourceFallbackDeps {
	/** Ask the core to continue a source, through the session controller's queue. */
	switchEngine: (request: SourceSwitch) => Promise<unknown>;
	context: () => FallbackContext;
}

export function createSourceFallback({ switchEngine, context }: SourceFallbackDeps) {
	/** Sources that ended on a failure Whisper can take over from, in the order they ended. */
	let ended = $state<Origin[]>([]);
	/** Sources reconnecting for `RECONNECT_OFFER_MS` and still at it. */
	let reconnecting = $state<Origin[]>([]);
	let switching = $state(false);
	/** `${origin}:${engine}` already tried or declined this run: offered once, never in a loop. */
	let tried = $state<string[]>([]);
	const timers = new Map<Origin, ReturnType<typeof setTimeout>>();

	const offer = $derived(whisperOffer(context()));
	const waiting = $derived(
		ORIGINS.filter(
			(origin) =>
				(ended.includes(origin) || reconnecting.includes(origin)) &&
				!!offer &&
				!tried.includes(`${origin}:${offer.to}`)
		)
	);

	function stopTimer(origin: Origin) {
		clearTimeout(timers.get(origin));
		timers.delete(origin);
	}

	function forget(origin: Origin) {
		stopTimer(origin);
		ended = ended.filter((o) => o !== origin);
		reconnecting = reconnecting.filter((o) => o !== origin);
	}

	function reset() {
		for (const origin of [...timers.keys()]) stopTimer(origin);
		ended = [];
		reconnecting = [];
		tried = [];
		switching = false;
	}

	/** A held run nothing will continue goes back to ending as it always did. */
	function releaseIfNothingToOffer() {
		if (get(heldSince) === null || get(sourcesActive)) return;
		const now = whisperOffer(context());
		const open = ended.some((origin) => now && !tried.includes(`${origin}:${now.to}`));
		if (!open) endHeldRun();
	}

	function noteStatus(status: StatusUpdate) {
		const { origin } = status;
		if (!origin) {
			if (status.state === 'idle') reset();
			return;
		}
		// One caption language only (v1): a lane status is about the source.
		if (endsForFallback(status)) {
			forget(origin);
			ended = [...ended, origin];
		} else if (status.state === 'reconnecting') {
			if (!timers.has(origin) && !reconnecting.includes(origin))
				timers.set(
					origin,
					setTimeout(() => {
						timers.delete(origin);
						reconnecting = [...reconnecting, origin];
					}, RECONNECT_OFFER_MS)
				);
		} else {
			forget(origin);
		}
		releaseIfNothingToOffer();
	}

	/** Continue every waiting source with offline Whisper, one call each, in order. */
	async function proceed() {
		const chosen = offer;
		if (!chosen || switching || waiting.length === 0) return;
		switching = true;
		const setup = context().setup;
		let continued = false;
		try {
			for (const origin of waiting) {
				const from = get(trackEngines)[origin] ?? setup.provider;
				const request: SourceSwitch = {
					origin,
					from,
					to: chosen.to,
					whisperModel: chosen.model,
					whisperGpu: chosen.gpu,
					reconnecting: reconnecting.includes(origin)
				};
				const key = `${origin}:${chosen.to}`;
				tried = [...tried, key];
				try {
					// The queue is busy with a Start or Stop, which settles the run itself.
					if ((await switchEngine(request)) === false) {
						tried = tried.filter((k) => k !== key);
						break;
					}
				} catch (error) {
					forget(origin);
					statusMessage.set(asStatus(error));
					continue;
				}
				continued = true;
				forget(origin);
				trackEngines.update((engines) => ({ ...engines, [origin]: chosen.to }));
				// Labelled in the interface language of the moment, as a marker typed then would be.
				const m = get(t);
				const started = get(sessionStartedAt);
				addMarker(
					m.fallback.marker(m.source[origin], m.fallback.offlineWhisper),
					started === null ? 0 : Date.now() - started
				);
				statusMessage.set((words) =>
					words.fallback.continued(words.source[origin], words.fallback.offlineWhisper)
				);
			}
		} finally {
			switching = false;
		}
		// A source continued resumes a held run when it reports; one refused everywhere ends it.
		if (!continued) releaseIfNothingToOffer();
	}

	/** Not now: the waiting sources stay as they are, and a held run ends at its failure. */
	function dismiss() {
		const chosen = offer;
		if (chosen) tried = [...tried, ...waiting.map((origin) => `${origin}:${chosen.to}`)];
		for (const origin of waiting) forget(origin);
		endHeldRun();
	}

	return {
		/** What the banner offers, or null when it has nothing to offer. */
		get offer() {
			return waiting.length > 0 ? offer : null;
		},
		/** The sources it would continue, in the order the core is asked. */
		get origins() {
			return waiting;
		},
		/** Whether `origin` is waiting because it is still reconnecting, rather than ended. */
		reconnecting(origin: Origin) {
			return reconnecting.includes(origin);
		},
		get switching() {
			return switching;
		},
		noteStatus,
		proceed,
		dismiss,
		dispose: reset
	};
}
