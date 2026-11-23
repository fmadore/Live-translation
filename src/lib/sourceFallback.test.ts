import { get } from 'svelte/store';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
	RECONNECT_OFFER_MS,
	createSourceFallback,
	whisperOffer,
	type FallbackContext
} from './sourceFallback.svelte';
import {
	applyStatus,
	beginSession,
	clearTranscript,
	describeStatus,
	heldSince,
	isRunning,
	markers,
	options,
	statusMessage,
	trackEngines
} from './stores';
import { en, setLocale } from './i18n';
import { DEFAULT_START_OPTIONS } from './startOptions';
import type { SourceSwitch, StartOptions, StatusUpdate, WhisperModelId } from './types';

const translate: StartOptions = {
	...DEFAULT_START_OPTIONS,
	source: 'both',
	mode: 'translate',
	provider: 'gemini',
	targetLanguage: 'en'
};

function installed(...ids: WhisperModelId[]) {
	return ids.map((id) => ({
		id,
		bytes: 1,
		installed: true,
		downloading: false,
		downloadedBytes: 0,
		inUse: false
	}));
}

function context(over: Partial<FallbackContext> = {}): FallbackContext {
	return {
		setup: translate,
		models: installed('small'),
		cpuSupported: true,
		rehearsal: false,
		...over
	};
}

const stopped = (origin: 'microphone' | 'system'): StatusUpdate => ({
	state: 'error',
	origin,
	message: { id: 'error.providerStopped', detail: 'Gemini — quota' }
});

describe('what the banner offers', () => {
	it('keeps translating into English with a model that translates', () => {
		expect(whisperOffer(context())).toEqual({
			to: 'whisper-translate',
			model: 'small',
			gpu: true,
			note: { kind: 'keepsEnglish' }
		});
	});

	it('continues in the spoken language when only Large v3 Turbo is installed', () => {
		expect(whisperOffer(context({ models: installed('turbo') }))).toEqual({
			to: 'whisper',
			model: 'turbo',
			gpu: true,
			note: { kind: 'turboCannotTranslate' }
		});
	});

	it('continues in the spoken language for a caption language other than English', () => {
		const setup = { ...translate, targetLanguage: 'fr' as const, whisperGpu: false };
		expect(whisperOffer(context({ setup }))).toEqual({
			to: 'whisper',
			model: 'small',
			gpu: false,
			note: { kind: 'cannotTranslate', language: 'fr' }
		});
	});

	it('transcribes for a transcription, with no note', () => {
		const setup: StartOptions = { ...translate, mode: 'transcribe', provider: 'mistral' };
		expect(whisperOffer(context({ setup, models: installed('tiny', 'turbo') }))).toEqual({
			to: 'whisper',
			model: 'turbo',
			gpu: true,
			note: null
		});
	});

	it('uses the setup’s model when it is installed and fits, else the best installed one', () => {
		const setup = { ...translate, whisperModel: 'base' as const };
		expect(whisperOffer(context({ setup, models: installed('base', 'small') }))?.model).toBe(
			'base'
		);
		expect(whisperOffer(context({ setup, models: installed('tiny', 'small') }))?.model).toBe(
			'small'
		);
		const turbo = { ...translate, whisperModel: 'turbo' as const };
		expect(
			whisperOffer(context({ setup: turbo, models: installed('turbo', 'tiny') }))
		).toMatchObject({ to: 'whisper-translate', model: 'tiny' });
	});

	it('offers nothing it cannot start', () => {
		const downloading = installed('small').map((m) => ({ ...m, downloading: true }));
		expect(whisperOffer(context({ models: [] }))).toBeNull();
		expect(whisperOffer(context({ models: downloading }))).toBeNull();
		expect(whisperOffer(context({ cpuSupported: false }))).toBeNull();
		expect(whisperOffer(context({ rehearsal: true }))).toBeNull();
		const demo: StartOptions = { ...translate, provider: 'ondevice', mode: 'transcribe' };
		expect(whisperOffer(context({ setup: demo }))).toBeNull();
		const local: StartOptions = { ...translate, provider: 'whisper-translate' };
		expect(whisperOffer(context({ setup: local }))).toBeNull();
		const dual: StartOptions = { ...translate, targetLanguage: 'fr', secondTargetLanguage: 'de' };
		expect(whisperOffer(context({ setup: dual }))).toBeNull();
	});
});

describe('the fallback controller', () => {
	beforeEach(() => {
		setLocale('en');
		options.set({ ...translate });
		beginSession(translate);
		applyStatus({ state: 'running', origin: 'microphone' });
		applyStatus({ state: 'running', origin: 'system' });
	});
	afterEach(() => {
		applyStatus({ state: 'idle' });
		clearTranscript();
		options.set({ ...DEFAULT_START_OPTIONS });
		vi.useRealTimers();
	});

	function controller(
		ctx = context(),
		switchEngine = vi.fn(async (_request: SourceSwitch): Promise<unknown> => true)
	) {
		const fallback = createSourceFallback({ switchEngine, context: () => ctx });
		const report = (status: StatusUpdate) => {
			applyStatus(status);
			fallback.noteStatus(status);
		};
		return { fallback, switchEngine, report };
	}

	it('offers a source the provider stopped or rejected, and not one whose capture failed', () => {
		const { fallback, report } = controller();
		report({ state: 'error', origin: 'microphone', message: { id: 'error.micStream' } });
		expect(fallback.offer).toBeNull();
		report({ state: 'error', origin: 'system', message: { id: 'error.providerRejected' } });
		expect(fallback.origins).toEqual(['system']);
		expect(fallback.offer?.to).toBe('whisper-translate');
	});

	it('continues both sources with one click, one call each, in order, and marks it once each', async () => {
		const { fallback, switchEngine, report } = controller();
		report(stopped('system'));
		report(stopped('microphone'));
		expect(get(heldSince)).not.toBeNull();
		await fallback.proceed();
		expect(switchEngine.mock.calls.map(([request]) => request)).toEqual<SourceSwitch[]>([
			{
				origin: 'microphone',
				from: 'gemini',
				to: 'whisper-translate',
				whisperModel: 'small',
				whisperGpu: true,
				reconnecting: false
			},
			{
				origin: 'system',
				from: 'gemini',
				to: 'whisper-translate',
				whisperModel: 'small',
				whisperGpu: true,
				reconnecting: false
			}
		]);
		expect(get(trackEngines)).toEqual({
			microphone: 'whisper-translate',
			system: 'whisper-translate'
		});
		expect(get(markers).map((marker) => marker.label)).toEqual([
			'System audio: offline Whisper from here',
			'Room mic: offline Whisper from here'
		]);
		expect(fallback.offer).toBeNull();
		// Still held until Whisper reports, then the run goes on.
		expect(get(isRunning)).toBe(true);
		report({ state: 'running', origin: 'microphone' });
		expect(get(heldSince)).toBeNull();
		expect(describeStatus(get(statusMessage), en, 'en')).toBe(
			'System audio continues with offline Whisper.'
		);
	});

	it('removes the offer a refused call leaves, says why, and ends a held run', async () => {
		const switchEngine = vi.fn(async (_request: SourceSwitch): Promise<unknown> => {
			throw { id: 'error.sessionFallback', detail: 'the source is still running' };
		});
		const { fallback, report } = controller(context(), switchEngine);
		report(stopped('microphone'));
		report(stopped('system'));
		await fallback.proceed();
		expect(switchEngine).toHaveBeenCalledTimes(2);
		expect(fallback.offer).toBeNull();
		expect(get(statusMessage)).toEqual({
			id: 'error.sessionFallback',
			detail: 'the source is still running'
		});
		expect(get(heldSince)).toBeNull();
		expect(get(isRunning)).toBe(false);
	});

	it('never offers an engine already tried for that source in this run', async () => {
		const { fallback, report } = controller();
		report(stopped('system'));
		await fallback.proceed();
		report({ state: 'running', origin: 'system' });
		// Whisper fails later on: an ordinary error, with nothing offered again.
		report({ state: 'error', origin: 'system', message: { id: 'error.providerStopped' } });
		expect(fallback.offer).toBeNull();
	});

	it('lets a held run end as before when there is nothing to offer', () => {
		const { fallback, report } = controller(context({ models: [] }));
		report(stopped('microphone'));
		report(stopped('system'));
		expect(fallback.offer).toBeNull();
		expect(get(isRunning)).toBe(false);
	});

	it('Not now leaves the sources as they are and ends the held run', () => {
		const { fallback, report } = controller();
		report(stopped('microphone'));
		report(stopped('system'));
		fallback.dismiss();
		expect(fallback.offer).toBeNull();
		expect(get(isRunning)).toBe(false);
	});

	it('offers a source still reconnecting after 30 seconds, beside the reconnection', async () => {
		vi.useFakeTimers();
		const { fallback, switchEngine, report } = controller();
		report({ state: 'reconnecting', origin: 'system' });
		vi.advanceTimersByTime(RECONNECT_OFFER_MS - 1);
		expect(fallback.offer).toBeNull();
		vi.advanceTimersByTime(1);
		expect(fallback.origins).toEqual(['system']);
		expect(fallback.reconnecting('system')).toBe(true);
		await fallback.proceed();
		expect(switchEngine).toHaveBeenCalledWith(
			expect.objectContaining({ origin: 'system', reconnecting: true })
		);
	});

	it.each([true, false])(
		'settles a failed reconnect switch with a live peer: %s',
		async (peerLive) => {
			vi.useFakeTimers();
			const error = { id: 'error.whisperModel' };
			const switchEngine = vi.fn(async (request: SourceSwitch) => {
				report({ state: 'error', origin: request.origin, message: error });
				throw error;
			});
			const { fallback, report } = controller(context(), switchEngine);
			if (!peerLive) report({ state: 'idle', origin: 'microphone' });
			report({ state: 'reconnecting', origin: 'system' });
			vi.advanceTimersByTime(RECONNECT_OFFER_MS);
			await fallback.proceed();
			expect(switchEngine).toHaveBeenCalledOnce();
			expect(fallback.offer).toBeNull();
			expect(get(isRunning)).toBe(peerLive);
			expect(get(statusMessage)).toEqual(error);
		}
	);

	it('withdraws the reconnect offer when the source comes back, and starts again on the next outage', () => {
		vi.useFakeTimers();
		const { fallback, report } = controller();
		report({ state: 'reconnecting', origin: 'system' });
		vi.advanceTimersByTime(RECONNECT_OFFER_MS / 2);
		report({ state: 'running', origin: 'system' });
		vi.advanceTimersByTime(RECONNECT_OFFER_MS);
		expect(fallback.offer).toBeNull();
		report({ state: 'reconnecting', origin: 'system' });
		vi.advanceTimersByTime(RECONNECT_OFFER_MS);
		expect(fallback.origins).toEqual(['system']);
		report({ state: 'running', origin: 'system' });
		expect(fallback.offer).toBeNull();
	});
});
