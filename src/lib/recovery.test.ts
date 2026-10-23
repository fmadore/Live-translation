import { afterEach, describe, expect, it, vi } from 'vitest';
import { describeError } from './errors';
import { en } from './i18n/en';
import { get } from 'svelte/store';
import {
	createRecoveryCoordinator,
	RECOVERY_INTERVAL_MS,
	RECOVERY_MAX_BACKOFF_MS,
	recoveryFailure,
	spoolFailure,
	startRecoverySpool
} from './recovery';
import type { TranscriptLine } from './types';

const lines: TranscriptLine[] = [{ id: 1, text: 'caption', sourceText: '', origin: 'microphone' }];
const port = () => ({
	writeRecovery: vi.fn().mockResolvedValue('snapshot.json'),
	clearRecovery: vi.fn().mockResolvedValue(undefined),
	readRecovery: vi.fn().mockResolvedValue(null)
});

afterEach(() => vi.useRealTimers());

describe('recovery coordination', () => {
	it('waits for an active write, skips queued obsolete text, then deletes', async () => {
		const io = port();
		let finish!: (path: string) => void;
		io.writeRecovery.mockReturnValueOnce(
			new Promise<string>((resolve) => {
				finish = resolve;
			})
		);
		const recovery = createRecoveryCoordinator(io);
		const active = recovery.write(lines);
		await Promise.resolve();
		const obsolete = recovery.write([{ ...lines[0], id: 2 }]);
		const clear = recovery.clear();
		expect(io.clearRecovery).not.toHaveBeenCalled();
		finish('snapshot.json');
		await Promise.all([active, obsolete, clear]);
		expect(io.writeRecovery).toHaveBeenCalledTimes(1);
		expect(io.clearRecovery).toHaveBeenCalledTimes(1);
	});

	it('can protect the same unsaved revision again after deletion', async () => {
		const io = port();
		const recovery = createRecoveryCoordinator(io);
		await recovery.write(lines);
		await recovery.write(lines);
		expect(io.writeRecovery).toHaveBeenCalledTimes(1);
		await recovery.clear();
		await recovery.write(lines);
		expect(io.writeRecovery).toHaveBeenCalledTimes(2);
	});

	it('a failed write does not prevent deletion or a later retry', async () => {
		const io = port();
		io.writeRecovery.mockRejectedValueOnce(new Error('disk full'));
		const recovery = createRecoveryCoordinator(io);
		await expect(recovery.write(lines)).rejects.toThrow('disk full');
		await recovery.clear();
		await recovery.write(lines);
		expect(io.clearRecovery).toHaveBeenCalledTimes(1);
		expect(io.writeRecovery).toHaveBeenCalledTimes(2);
	});

	it('disabling the timer and clearing cannot leave its active write behind', async () => {
		vi.useFakeTimers();
		const io = port();
		let finish!: (path: string) => void;
		io.writeRecovery.mockReturnValueOnce(
			new Promise<string>((resolve) => {
				finish = resolve;
			})
		);
		const recovery = createRecoveryCoordinator(io);
		const onError = vi.fn();
		const stop = startRecoverySpool(() => lines, onError, recovery);
		await vi.advanceTimersByTimeAsync(RECOVERY_INTERVAL_MS);
		stop();
		const clear = recovery.clear();
		finish('snapshot.json');
		await clear;
		await vi.advanceTimersByTimeAsync(RECOVERY_INTERVAL_MS * 3);
		expect(io.writeRecovery).toHaveBeenCalledTimes(1);
		expect(io.clearRecovery).toHaveBeenCalledTimes(1);
		expect(onError).not.toHaveBeenCalled();
	});
});

describe('a failed spool write', () => {
	// The structure survives, so the status line words it in whichever language is showing.
	it('passes on what the core reported', async () => {
		vi.useFakeTimers();
		const io = port();
		const failure = { id: 'error.recoveryWrite', detail: 'C:\\recovery — disk full' };
		io.writeRecovery.mockRejectedValueOnce(failure);
		const onError = vi.fn();
		const stop = startRecoverySpool(() => lines, onError, createRecoveryCoordinator(io));
		await vi.advanceTimersByTimeAsync(RECOVERY_INTERVAL_MS);
		stop();
		expect(onError).toHaveBeenCalledWith(failure);
		expect(spoolFailure(failure)).toBe(failure);
	});

	// Review U3: the first failure used to stop the spool for good, with one message on a status
	// line that the next status overwrote.
	it('keeps trying, further apart each time, and reports once per run of failures', async () => {
		vi.useFakeTimers();
		const io = port();
		const failure = { id: 'error.recoveryWrite', detail: 'disk full' };
		io.writeRecovery
			.mockRejectedValueOnce(failure)
			.mockRejectedValueOnce(failure)
			.mockRejectedValueOnce(failure);
		const onError = vi.fn();
		let unsaved = lines;
		const stop = startRecoverySpool(() => unsaved, onError, createRecoveryCoordinator(io));
		try {
			await vi.advanceTimersByTimeAsync(RECOVERY_INTERVAL_MS);
			expect(io.writeRecovery).toHaveBeenCalledTimes(1);
			expect(get(recoveryFailure)).toEqual(failure);
			// Twice the interval before the next attempt, then four times, then the cap.
			await vi.advanceTimersByTimeAsync(RECOVERY_INTERVAL_MS);
			expect(io.writeRecovery).toHaveBeenCalledTimes(1);
			await vi.advanceTimersByTimeAsync(RECOVERY_INTERVAL_MS);
			expect(io.writeRecovery).toHaveBeenCalledTimes(2);
			await vi.advanceTimersByTimeAsync(RECOVERY_INTERVAL_MS * 4);
			expect(io.writeRecovery).toHaveBeenCalledTimes(3);
			await vi.advanceTimersByTimeAsync(RECOVERY_MAX_BACKOFF_MS - RECOVERY_INTERVAL_MS);
			expect(io.writeRecovery).toHaveBeenCalledTimes(3);
			await vi.advanceTimersByTimeAsync(RECOVERY_INTERVAL_MS);
			expect(io.writeRecovery).toHaveBeenCalledTimes(4);
			// The fourth went through: the run is over, and the next tick is on time again.
			expect(onError).toHaveBeenCalledOnce();
			expect(get(recoveryFailure)).toBe('');

			// A new failure is a new run, reported again.
			unsaved = [{ ...lines[0], id: 2 }];
			io.writeRecovery.mockRejectedValueOnce(new Error('IPC closed'));
			await vi.advanceTimersByTimeAsync(RECOVERY_INTERVAL_MS);
			expect(io.writeRecovery).toHaveBeenCalledTimes(5);
			expect(onError).toHaveBeenCalledTimes(2);
			expect(get(recoveryFailure)).toEqual({ id: 'error.recoveryWrite', detail: 'IPC closed' });
		} finally {
			stop();
		}
		// Switched off, nothing is failing.
		expect(get(recoveryFailure)).toBe('');
	});

	it('stops counting as failing once nothing is left unsaved to protect', async () => {
		vi.useFakeTimers();
		const io = port();
		io.writeRecovery.mockRejectedValueOnce(new Error('locked'));
		let unsaved: TranscriptLine[] | null = lines;
		const stop = startRecoverySpool(() => unsaved, vi.fn(), createRecoveryCoordinator(io));
		try {
			await vi.advanceTimersByTimeAsync(RECOVERY_INTERVAL_MS);
			expect(get(recoveryFailure)).not.toBe('');
			unsaved = null;
			await vi.advanceTimersByTimeAsync(RECOVERY_INTERVAL_MS * 2);
			expect(get(recoveryFailure)).toBe('');
		} finally {
			stop();
		}
	});

	it('is still reported as the spool’s when something else stopped it', () => {
		expect(spoolFailure(new Error('IPC closed'))).toEqual({
			id: 'error.recoveryWrite',
			detail: 'IPC closed'
		});
		expect(describeError(spoolFailure('IPC closed'), en)).toBe(
			'The recovery copy could not be written (IPC closed)'
		);
	});
});
