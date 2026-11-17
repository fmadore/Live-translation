// The live rail's notice for the overlay: a suggestion in one click, or a line of the
// operator's own, and the way to take it down.

import { beforeEach, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import OverlayNoticeControl from './OverlayNoticeControl.svelte';
import { createOverlayController } from './overlayController.svelte';
import { operatorContextMap } from './operatorContext';
import { noticeUntilCaptions, options } from './stores';
import { DEFAULT_START_OPTIONS } from './startOptions';
import { setLocale } from './i18n';
import type { OverlayConfig } from './types';

function mount() {
	const pushes: Partial<OverlayConfig>[] = [];
	const overlay = createOverlayController(
		{ setOverlayConfig: async (c: OverlayConfig) => void pushes.push(c) } as never,
		(push) => push()
	);
	render(OverlayNoticeControl, { context: operatorContextMap({ overlay }) });
	return { overlay, pushes };
}

beforeEach(() => {
	setLocale('en');
	noticeUntilCaptions.set(true);
	options.set({ ...DEFAULT_START_OPTIONS, mode: 'translate', targetLanguage: 'de' });
});

it('puts a suggestion up in the room’s language in one click, says so, and takes it down', async () => {
	const { overlay, pushes } = mount();
	const opener = screen.getByRole('button', { name: 'Show a notice on the overlay…' });
	await fireEvent.click(opener);
	await waitFor(() =>
		expect(document.activeElement).toBe(screen.getByRole('textbox', { name: 'Notice text' }))
	);
	// The captions are in German, so the suggestions are too.
	const group = screen.getByRole('group', { name: 'Suggestions' });
	expect(group).toHaveTextContent('Gleich geht es weiter');
	await fireEvent.click(screen.getByRole('button', { name: 'Untertitel pausiert' }));
	expect(overlay.notice).toBe('Untertitel pausiert');
	await waitFor(() => expect(pushes.at(-1)?.notice).toBe('Untertitel pausiert'));
	const status = screen.getByRole('status');
	expect(status).toHaveTextContent('The overlay shows “Untertitel pausiert” instead of captions.');
	await fireEvent.click(screen.getByRole('button', { name: 'Clear notice' }));
	expect(overlay.notice).toBe('');
	await waitFor(() => expect(status).toHaveTextContent('The overlay shows captions again.'));
});

it('shows a line of the operator’s own, and keeps the choice to wait for captions', async () => {
	const { overlay } = mount();
	await fireEvent.click(screen.getByRole('button', { name: 'Show a notice on the overlay…' }));
	const show = screen.getByRole('button', { name: 'Show notice' });
	expect(show).toBeDisabled();
	const until = screen.getByRole('checkbox', {
		name: 'Remove it when captions resume after a pause'
	});
	expect(until).toBeChecked();
	await fireEvent.click(until);
	expect(get(noticeUntilCaptions)).toBe(false);
	const field = screen.getByRole('textbox', { name: 'Notice text' });
	expect(field).toHaveAttribute('maxlength', '80');
	await fireEvent.input(field, { target: { value: 'Lunch until 1:30' } });
	await fireEvent.submit(field.closest('form')!);
	expect(overlay.notice).toBe('Lunch until 1:30');
});

it('puts the panel away on Escape, with focus back on its button', async () => {
	vi.useRealTimers();
	mount();
	await fireEvent.click(screen.getByRole('button', { name: 'Show a notice on the overlay…' }));
	await fireEvent.keyDown(screen.getByRole('textbox', { name: 'Notice text' }), { key: 'Escape' });
	const opener = screen.getByRole('button', { name: 'Show a notice on the overlay…' });
	await waitFor(() => expect(document.activeElement).toBe(opener));
});
