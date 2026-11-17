import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { CHECK_DELAY_MS, RETRY_DELAY_MS, createUpdateCheck } from './updateCheck.svelte';
import type { UpdateStatus } from './types';

const INELIGIBLE: UpdateStatus = {
	eligible: false,
	checks: 'unanswered',
	ask: false,
	lastCheckedMs: null,
	available: null
};
const ELIGIBLE: UpdateStatus = { ...INELIGIBLE, eligible: true };

/** A core whose answers the test sets, counting the calls that would reach GitHub. */
function fakeCore(initial: UpdateStatus) {
	let current = initial;
	let found: string | null = null;
	return {
		set(next: Partial<UpdateStatus>) {
			current = { ...current, ...next };
		},
		/** What the next check will find. */
		release(version: string) {
			found = version;
		},
		updateStatus: vi.fn(async () => current),
		checkForUpdate: vi.fn(async () => {
			if (current.eligible && current.checks === 'on') {
				current = { ...current, lastCheckedMs: 1000, available: found };
			}
			return current;
		}),
		setUpdateChecks: vi.fn(async (on: boolean) => {
			current = { ...current, checks: on ? 'on' : 'off', ask: false };
			return current;
		}),
		skipUpdateVersion: vi.fn(async () => {
			current = { ...current, available: null };
			return current;
		}),
		openReleasePage: vi.fn(async () => {})
	};
}

beforeEach(() => {
	vi.useFakeTimers();
});

afterEach(() => {
	vi.useRealTimers();
});

describe('createUpdateCheck', () => {
	it('shows nothing before the core has answered, nor for a copy that does not check', async () => {
		const core = fakeCore({ ...INELIGIBLE, available: '9.0.0', ask: true });
		const updates = createUpdateCheck(core);
		expect(updates.eligible).toBe(false);

		await updates.load();
		updates.schedule(() => false);
		await vi.advanceTimersByTimeAsync(CHECK_DELAY_MS * 2);

		expect(updates.eligible).toBe(false);
		expect(updates.available).toBeNull();
		expect(updates.ask).toBe(false);
		expect(core.checkForUpdate).not.toHaveBeenCalled();
	});

	it('checks a few seconds after launch, not before', async () => {
		const core = fakeCore({ ...ELIGIBLE, checks: 'on' });
		core.release('1.15.0');
		const updates = createUpdateCheck(core);
		await updates.load();
		updates.schedule(() => false);

		await vi.advanceTimersByTimeAsync(CHECK_DELAY_MS - 1);
		expect(core.checkForUpdate).not.toHaveBeenCalled();
		await vi.advanceTimersByTimeAsync(1);
		expect(core.checkForUpdate).toHaveBeenCalledOnce();
		expect(updates.available).toBe('1.15.0');
		expect(updates.lastCheckedMs).toBe(1000);
	});

	it('waits for a running session to end', async () => {
		const core = fakeCore({ ...ELIGIBLE, checks: 'on' });
		const updates = createUpdateCheck(core);
		await updates.load();
		let running = true;
		updates.schedule(() => running);

		await vi.advanceTimersByTimeAsync(CHECK_DELAY_MS + RETRY_DELAY_MS * 3);
		expect(core.checkForUpdate).not.toHaveBeenCalled();

		running = false;
		await vi.advanceTimersByTimeAsync(RETRY_DELAY_MS);
		expect(core.checkForUpdate).toHaveBeenCalledOnce();
	});

	it('asks from the core’s say-so, and Turn on answers it and checks at once', async () => {
		const core = fakeCore({ ...ELIGIBLE, ask: true });
		core.release('1.15.0');
		const updates = createUpdateCheck(core);
		await updates.load();
		expect(updates.ask).toBe(true);

		await updates.setChecks(true);

		expect(core.setUpdateChecks).toHaveBeenCalledWith(true);
		expect(updates.ask).toBe(false);
		expect(updates.checks).toBe('on');
		expect(core.checkForUpdate).toHaveBeenCalledOnce();
		expect(updates.available).toBe('1.15.0');
	});

	it('No thanks answers it without a request', async () => {
		const core = fakeCore({ ...ELIGIBLE, ask: true });
		const updates = createUpdateCheck(core);
		await updates.load();

		await updates.setChecks(false);

		expect(updates.ask).toBe(false);
		expect(updates.checks).toBe('off');
		expect(core.checkForUpdate).not.toHaveBeenCalled();
	});

	it('Not now hides the version', async () => {
		const core = fakeCore({ ...ELIGIBLE, checks: 'on', available: '1.15.0' });
		const updates = createUpdateCheck(core);
		await updates.load();
		expect(updates.available).toBe('1.15.0');

		await updates.notNow();

		expect(core.skipUpdateVersion).toHaveBeenCalledOnce();
		expect(updates.available).toBeNull();
	});

	it('is silent when the core fails', async () => {
		const core = fakeCore(ELIGIBLE);
		core.updateStatus.mockRejectedValueOnce({ id: 'error.taskFailed' });
		const updates = createUpdateCheck(core);
		await expect(updates.load()).resolves.toBeUndefined();
		expect(updates.eligible).toBe(false);
	});

	it('mounts the region before its sentence', async () => {
		const core = fakeCore({ ...ELIGIBLE, checks: 'on', available: '1.15.0' });
		let rendered = () => {};
		const updates = createUpdateCheck(
			core,
			() => new Promise<void>((resolve) => (rendered = resolve))
		);
		const loading = updates.load();
		await vi.waitFor(() => expect(updates.eligible).toBe(true));
		expect(updates.available).toBeNull();

		rendered();
		await loading;
		expect(updates.available).toBe('1.15.0');
	});

	it('stops waiting once disposed', async () => {
		const core = fakeCore({ ...ELIGIBLE, checks: 'on' });
		const updates = createUpdateCheck(core);
		await updates.load();
		updates.schedule(() => false);
		updates.dispose();
		await vi.advanceTimersByTimeAsync(CHECK_DELAY_MS * 2);
		expect(core.checkForUpdate).not.toHaveBeenCalled();
	});
});
