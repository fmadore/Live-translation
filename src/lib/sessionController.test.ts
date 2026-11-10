import { beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import { createSessionController } from './sessionController';
import {
	applyStatus,
	clearTranscript,
	describeStatus,
	isRunning,
	options,
	pauseRequested,
	sessionStartedAt,
	statusMessage
} from './stores';
import { locale, t } from './i18n';

beforeEach(() => {
	applyStatus({ state: 'idle' });
	clearTranscript();
});

/** The status line as the operator would read it now. */
function statusLine(): string {
	return describeStatus(get(statusMessage), get(t), get(locale));
}

describe('session coordination', () => {
	it('blocks unsupported targets before IPC or starting a transcript, in every interface locale', async () => {
		const startSession = vi.fn();
		const controller = createSessionController({
			startSession,
			stopSession: vi.fn(),
			pauseSession: vi.fn()
		});
		for (const language of ['en', 'fr', 'de'] as const) {
			locale.set(language);
			expect(
				await controller.start({
					...get(options),
					mode: 'translate',
					provider: 'openai',
					targetLanguage: 'sw'
				})
			).toBe(false);
			expect(statusLine()).toMatch(/OpenAI/);
			expect(statusLine()).toMatch(/swahili|Swahili|Suaheli/);
		}
		locale.set('en');
		expect(startSession).not.toHaveBeenCalled();
		expect(get(sessionStartedAt)).toBeNull();
	});

	// Review U11: worded where it is shown, so it follows a change of interface language rather
	// than staying in the one that was active when Start was refused.
	it('re-words a refused start when the interface language changes', async () => {
		const controller = createSessionController({
			startSession: vi.fn(),
			stopSession: vi.fn(),
			pauseSession: vi.fn()
		});
		locale.set('en');
		await controller.start({
			...get(options),
			mode: 'translate',
			provider: 'openai',
			targetLanguage: 'sw'
		});
		expect(statusLine()).toBe('OpenAI does not support Swahili — choose another language.');
		locale.set('de');
		expect(statusLine()).toMatch(/^OpenAI unterstützt .*Suaheli/);
		locale.set('en');
	});

	// Review U1 and B4: every source of a run failed, so the core still holds that session, and
	// the next start stops it first. Its whole-session Idle, carrying the old failure, is sent
	// before anything of the new run, but events and command answers travel separately: it can
	// land while the start is in flight or after the start call has answered.
	describe('the Idle of the session a start replaced', () => {
		const failure = { id: 'error.whisperSession', detail: 'decoder stalled' };

		/** A run that started and then ended by itself: the core still holds it. */
		async function endedByItself() {
			const stopSession = vi.fn().mockImplementation(async () => applyStatus({ state: 'idle' }));
			const startSession = vi.fn().mockResolvedValue(undefined);
			const controller = createSessionController({
				startSession,
				stopSession,
				pauseSession: vi.fn()
			});
			expect(await controller.start(get(options))).toBe(true);
			applyStatus({ state: 'running', origin: 'microphone' });
			applyStatus({ state: 'error', origin: 'microphone', message: failure });
			expect(get(isRunning)).toBe(false);
			statusMessage.set('');
			return { controller, startSession, stopSession };
		}

		it('is not taken for the new run ending when it arrives during the start', async () => {
			const { controller, startSession } = await endedByItself();
			startSession.mockImplementationOnce(async () =>
				applyStatus({ state: 'idle', message: failure })
			);
			expect(await controller.start(get(options))).toBe(true);
			expect(get(sessionStartedAt)).not.toBeNull();
			expect(get(statusMessage)).toBe('');
			applyStatus({ state: 'running', origin: 'microphone' });
			expect(get(isRunning)).toBe(true);
			await controller.stop();
			expect(get(isRunning)).toBe(false);
			expect(get(sessionStartedAt)).toBeNull();
		});

		it('is not taken for the new run ending when it arrives after the start call answered', async () => {
			const { controller } = await endedByItself();
			expect(await controller.start(get(options))).toBe(true);
			const startedAt = get(sessionStartedAt);
			applyStatus({ state: 'idle', message: failure });
			expect(get(sessionStartedAt)).toBe(startedAt);
			expect(get(statusMessage)).toBe('');
			applyStatus({ state: 'connecting', origin: 'microphone' });
			applyStatus({ state: 'running', origin: 'microphone' });
			expect(get(isRunning)).toBe(true);
			expect(get(sessionStartedAt)).toBe(startedAt);

			// Only one was due: the next whole-session Idle is this run's own end.
			applyStatus({ state: 'idle' });
			expect(get(isRunning)).toBe(false);
			expect(get(sessionStartedAt)).toBeNull();
		});

		it('still lets Stop end a run pressed straight after its Start', async () => {
			const { controller } = await endedByItself();
			expect(await controller.start(get(options))).toBe(true);
			// The old run's Idle, then Stop's own, before any source of the new run reported.
			applyStatus({ state: 'idle', message: failure });
			await controller.stop();
			expect(get(sessionStartedAt)).toBeNull();
			expect(get(statusMessage)).toBe('');
		});

		it('keeps a failed start’s reason when the old run’s Idle lands after it', async () => {
			const { controller, startSession } = await endedByItself();
			startSession.mockRejectedValueOnce(new Error('missing credential'));
			expect(await controller.start(get(options))).toBe(false);
			expect(get(sessionStartedAt)).toBeNull();
			applyStatus({ state: 'idle' });
			expect(statusLine()).toContain('missing credential');
			// That Idle was the one due; a later one is honoured again.
			applyStatus({ state: 'idle' });
			expect(get(statusMessage)).toBe('');
		});

		it('is not expected after a run the core answered with its own Idle', async () => {
			const { controller } = await endedByItself();
			await controller.stop();
			expect(await controller.start(get(options))).toBe(true);
			// Stop pressed before any source reported: its Idle ends the run at once.
			await controller.stop();
			expect(get(sessionStartedAt)).toBeNull();
		});
	});

	it('resets the clock when startup fails', async () => {
		const controller = createSessionController({
			startSession: vi.fn().mockRejectedValue(new Error('missing credential')),
			stopSession: vi.fn(),
			pauseSession: vi.fn()
		});
		expect(await controller.start(get(options))).toBe(false);
		expect(get(sessionStartedAt)).toBeNull();
		expect(get(statusMessage)).toContain('missing credential');
		expect(get(controller.busy)).toBe(false);
	});

	it('serializes Stop behind startup and coalesces repeated stops', async () => {
		let finish!: () => void;
		const startSession = vi.fn().mockReturnValue(
			new Promise<void>((resolve) => {
				finish = resolve;
			})
		);
		const stopSession = vi.fn().mockResolvedValue(undefined);
		const controller = createSessionController({
			startSession,
			stopSession,
			pauseSession: vi.fn()
		});
		const starting = controller.start(get(options));
		expect(await controller.start(get(options))).toBe(false);
		const stopping = controller.stop();
		expect(controller.stop()).toBe(stopping);
		expect(stopSession).not.toHaveBeenCalled();
		expect(get(controller.busy)).toBe(true);
		finish();
		await Promise.all([starting, stopping]);
		expect(startSession).toHaveBeenCalledTimes(1);
		expect(stopSession).toHaveBeenCalledTimes(1);
		expect(get(controller.busy)).toBe(false);
	});

	it('pauses and resumes a running session, and ignores a pause with none running', async () => {
		const pauseSession = vi.fn().mockResolvedValue(undefined);
		const controller = createSessionController({
			startSession: vi.fn(),
			stopSession: vi.fn(),
			pauseSession
		});
		await controller.pause(true);
		expect(pauseSession).not.toHaveBeenCalled();

		applyStatus({ state: 'running', origin: 'microphone' });
		await controller.pause(true);
		expect(pauseSession).toHaveBeenLastCalledWith(true);
		expect(get(pauseRequested)).toBe(true);
		await controller.pause(false);
		expect(get(pauseRequested)).toBe(false);

		pauseSession.mockRejectedValueOnce(new Error('gone'));
		await controller.pause(true);
		expect(get(pauseRequested)).toBe(false);
		expect(get(statusMessage)).toBeTruthy();
	});
});
