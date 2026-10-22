import { beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import { createSessionController } from './sessionController';
import {
	applyStatus,
	clearTranscript,
	describeStatus,
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

	// Review U1: the core stops the session it still holds before it starts the next, and that
	// session's Idle arrives while the start is in flight. The page reads this to tell them apart.
	it('says when a start is in flight, until the core answers it', async () => {
		let finish!: () => void;
		const controller = createSessionController({
			startSession: vi.fn().mockReturnValue(
				new Promise<void>((resolve) => {
					finish = resolve;
				})
			),
			stopSession: vi.fn(),
			pauseSession: vi.fn()
		});
		expect(get(controller.starting)).toBe(false);
		const starting = controller.start(get(options));
		expect(get(controller.starting)).toBe(true);
		finish();
		expect(await starting).toBe(true);
		expect(get(controller.starting)).toBe(false);
		applyStatus({ state: 'idle' });

		const failing = createSessionController({
			startSession: vi.fn().mockRejectedValue(new Error('no device')),
			stopSession: vi.fn(),
			pauseSession: vi.fn()
		});
		expect(await failing.start(get(options))).toBe(false);
		expect(get(failing.starting)).toBe(false);
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
