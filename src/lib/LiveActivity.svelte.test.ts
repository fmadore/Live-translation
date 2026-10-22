import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { render } from '@testing-library/svelte';
import LiveActivity from './LiveActivity.svelte';
import { setLocale } from './i18n';
import { activityTimes, applyStatus, beginSession, originStates, runningSince } from './stores';

const now = 1_000_000;

beforeEach(() => {
	setLocale('en');
	originStates.set({ system: 'running', microphone: 'running' });
	runningSince.set({ system: now - 20_000, microphone: now - 20_000 });
	activityTimes.set({});
});
afterEach(() => {
	vi.useRealTimers();
	applyStatus({ state: 'idle' });
});

function live() {
	return render(LiveActivity, { props: { now, microphone: true, system: true } });
}

// Review D21: the whole panel was the live region, and its labels flip with every pause in
// speech, so Narrator could read it out again and again. Only going wrong is announced now.
it('shows every source but announces nothing while they are well', () => {
	activityTimes.set({ system: { audio: now - 1000, caption: now - 1000 } });
	const view = live();
	expect(view.getByRole('group', { name: 'Live input status' })).toHaveTextContent(
		/Remote · Receiving captions/
	);
	expect(view.getByRole('status')).toHaveTextContent(/^$/);
	view.unmount();
});

it('announces a source that has gone quiet on captions or stopped with an error', async () => {
	// Audio for 20 seconds and not one caption: the stale state.
	activityTimes.set({ system: { audio: now - 1000, caption: 0 } });
	const view = live();
	expect(view.getByRole('status')).toHaveTextContent(/no captions have arrived for 15 seconds/);
	expect(view.getByRole('status')).not.toHaveTextContent(/Room/);

	originStates.set({ system: 'running', microphone: 'error' });
	await Promise.resolve();
	expect(view.getByRole('status')).toHaveTextContent(/Source stopped with an error/);
	view.unmount();
});

// Review U6: capture keeps metering while a source is paused, so a break longer than 15 seconds
// raised the alert, and Narrator spoke it, the moment the source resumed.
it('counts the wait for captions from the resume, not from before the break', async () => {
	vi.useFakeTimers({ toFake: ['Date'] });
	vi.setSystemTime(now - 300_000);
	beginSession();
	applyStatus({ state: 'running', origin: 'system' });
	vi.setSystemTime(now - 240_000);
	applyStatus({ state: 'paused', origin: 'system' });
	// Back from a four-minute break: the room is talking, and the first caption is not in yet.
	vi.setSystemTime(now - 5_000);
	applyStatus({ state: 'running', origin: 'system' });
	activityTimes.set({ system: { audio: now - 500, caption: now - 250_000 } });

	const view = render(LiveActivity, { props: { now, microphone: false, system: true } });
	expect(view.getByRole('group')).toHaveTextContent(/Remote · Receiving audio/);
	expect(view.getByRole('status')).toHaveTextContent(/^$/);

	// Fifteen seconds after the resume with audio and still no caption: that is worth saying.
	activityTimes.set({ system: { audio: now + 10_000, caption: now - 250_000 } });
	await view.rerender({ now: now + 10_500 });
	expect(view.getByRole('status')).toHaveTextContent(/no captions have arrived for 15 seconds/);
	view.unmount();
});
