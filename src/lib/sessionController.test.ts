import { beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import { createSessionController } from './sessionController';
import { applyStatus, clearTranscript, options, sessionStartedAt, statusMessage } from './stores';

beforeEach(() => {
	applyStatus({ state: 'idle' });
	clearTranscript();
});

describe('session coordination', () => {
	it('resets the clock when startup fails', async () => {
		const controller = createSessionController({
			startSession: vi.fn().mockRejectedValue(new Error('missing credential')),
			stopSession: vi.fn()
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
		const controller = createSessionController({ startSession, stopSession });
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
});
