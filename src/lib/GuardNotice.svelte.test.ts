// The spending guard as the operator meets it: its three choices in Settings › App, and its
// notices above the stage.

import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import { fireEvent, render, screen } from '@testing-library/svelte';
import GuardNotice from './GuardNotice.svelte';
import SpendingGuardSettings from './SpendingGuardSettings.svelte';
import { costWarningUsd, durationWarningHours, idlePauseMinutes } from './spendingGuard';
import type { SpendingGuard } from './spendingGuard.svelte';
import { setLocale } from './i18n';

beforeEach(() => setLocale('en'));
afterEach(() => {
	idlePauseMinutes.set(0);
	costWarningUsd.set(0);
	durationWarningHours.set(0);
});

it('offers each choice off by default, and keeps what is picked', async () => {
	render(SpendingGuardSettings);
	const idle = screen.getByRole('combobox', { name: 'Pause when nothing has been captioned for' });
	const cost = screen.getByRole('combobox', { name: 'Tell me when the estimated cost passes' });
	const hours = screen.getByRole('combobox', { name: 'Tell me when a session has run for' });
	for (const select of [idle, cost, hours]) expect(select).toHaveDisplayValue('Never');
	await fireEvent.change(idle, { target: { value: '15' } });
	await fireEvent.change(cost, { target: { value: '5' } });
	await fireEvent.change(hours, { target: { value: '2' } });
	expect(get(idlePauseMinutes)).toBe(15);
	expect(get(costWarningUsd)).toBe(5);
	expect(get(durationWarningHours)).toBe(2);
	expect(cost).toHaveDisplayValue('$5.00');
	expect(hours).toHaveDisplayValue('2 hours');
	expect(idle).toHaveAccessibleDescription(/Cloud engines only/);
});

function guard(state: Partial<SpendingGuard>): SpendingGuard {
	return {
		idlePaused: null,
		costPassed: null,
		durationPassed: null,
		dismiss: vi.fn(),
		...state
	} as SpendingGuard;
}

it('says why the session was paused, with Resume and the overlay notice beside it', async () => {
	const onResume = vi.fn();
	const onShowNotice = vi.fn();
	const g = guard({ idlePaused: 10 });
	const view = render(GuardNotice, {
		props: {
			guard: g,
			overlayNoticeShown: false,
			onResume,
			pausedNotice: 'Captions paused',
			onShowNotice
		}
	});
	// Said by the status region, and drawn beside its buttons.
	expect(screen.getByRole('status')).toHaveTextContent(/nothing was captioned for 10 minutes/);
	expect(screen.getAllByText(/nothing was captioned for 10 minutes/)).toHaveLength(2);
	await fireEvent.click(screen.getByRole('button', { name: 'Resume' }));
	expect(onResume).toHaveBeenCalledOnce();
	await fireEvent.click(
		screen.getByRole('button', { name: 'Show “Captions paused” on the overlay' })
	);
	expect(onShowNotice).toHaveBeenCalledOnce();
	await fireEvent.click(screen.getByRole('button', { name: 'Dismiss' }));
	expect(g.dismiss).toHaveBeenCalledWith('idle');
	// With a notice already up, the offer is not made again.
	await view.rerender({ overlayNoticeShown: true });
	expect(screen.queryByRole('button', { name: /on the overlay/ })).toBeNull();
});

it('says once the estimate and the length have passed, and offers only to dismiss them', async () => {
	const g = guard({ costPassed: 5, durationPassed: 1 });
	render(GuardNotice, {
		props: {
			guard: g,
			overlayNoticeShown: false,
			onResume: vi.fn(),
			pausedNotice: 'Captions paused',
			onShowNotice: vi.fn()
		}
	});
	const status = screen.getByRole('status');
	expect(status).toHaveTextContent(/has passed \$5\.00\. It is still running\./);
	expect(status).toHaveTextContent(/running for an hour/);
	expect(screen.queryByRole('button', { name: 'Resume' })).toBeNull();
	const [first, second] = screen.getAllByRole('button', { name: 'Dismiss' });
	await fireEvent.click(first);
	await fireEvent.click(second);
	expect(g.dismiss).toHaveBeenCalledWith('cost');
	expect(g.dismiss).toHaveBeenCalledWith('duration');
});

it('has its status region in place and silent while the guard has nothing to say', () => {
	render(GuardNotice, {
		props: {
			guard: guard({}),
			overlayNoticeShown: false,
			onResume: vi.fn(),
			pausedNotice: 'Captions paused',
			onShowNotice: vi.fn()
		}
	});
	expect(screen.getByRole('status').textContent).toBe('');
	expect(screen.queryByRole('button')).toBeNull();
});
