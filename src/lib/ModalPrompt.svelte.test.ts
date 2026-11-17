import { describe, expect, it, vi } from 'vitest';
import { createRawSnippet } from 'svelte';
import { fireEvent, render } from '@testing-library/svelte';
import ModalPrompt from './ModalPrompt.svelte';
import UnsavedPrompt from './UnsavedPrompt.svelte';

// Settings stays open when the window's X or the tray's Quit raises the unsaved prompt, so two
// dialogs listen on the window at once. Only the newer one may answer the keyboard.
function stack() {
	const onSettingsDismiss = vi.fn();
	render(ModalPrompt, {
		props: {
			title: 'Settings',
			onDismiss: onSettingsDismiss,
			children: createRawSnippet(() => ({ render: () => '<button>Inside settings</button>' }))
		}
	});
	const onChoice = vi.fn();
	const view = render(UnsavedPrompt, {
		props: { lines: 3, endedSession: false, answering: null, error: '', onChoice }
	});
	return { ...view, onSettingsDismiss, onChoice };
}

describe('ModalPrompt stacking', () => {
	it('lets Escape answer only the topmost dialog', async () => {
		const { onSettingsDismiss, onChoice } = stack();

		await fireEvent.keyDown(window, { key: 'Escape' });

		expect(onChoice).toHaveBeenCalledWith('cancel');
		expect(onSettingsDismiss).not.toHaveBeenCalled();
	});

	it('leaves Tab inside the topmost dialog to the browser', async () => {
		const { getByRole } = stack();
		const discard = getByRole('button', { name: 'Discard and close' });
		discard.focus();

		// Mid-cycle Tab is not the dialog's to handle. The dialog underneath used to treat
		// focus outside itself as escaped and pull it back, so Discard could not be reached.
		await fireEvent.keyDown(window, { key: 'Tab' });

		expect(document.activeElement).toBe(discard);
	});

	it('hands the keyboard back when the top dialog closes', async () => {
		const { onSettingsDismiss, unmount } = stack();

		unmount();
		await fireEvent.keyDown(window, { key: 'Escape' });

		expect(onSettingsDismiss).toHaveBeenCalledOnce();
	});
});

describe('ModalPrompt initial focus', () => {
	const body = createRawSnippet(() => ({
		render: () =>
			'<div><button>First</button><input aria-label="Wanted" class="wanted" /><input aria-label="Off" class="off" disabled /></div>'
	}));

	it('focuses the first control by default, which is the corner Close when there is one', () => {
		const view = render(ModalPrompt, {
			props: { title: 'Panel', onDismiss: vi.fn(), dismissLabel: 'Close', children: body }
		});
		expect(view.getByRole('button', { name: 'Close' })).toHaveFocus();
	});

	it('focuses what initialFocus names instead', () => {
		const view = render(ModalPrompt, {
			props: {
				title: 'Panel',
				onDismiss: vi.fn(),
				dismissLabel: 'Close',
				initialFocus: '.wanted',
				children: body
			}
		});
		expect(view.getByLabelText('Wanted')).toHaveFocus();
	});

	it('falls back to the first control when initialFocus names a disabled or missing one', () => {
		for (const initialFocus of ['.off', '.absent']) {
			const view = render(ModalPrompt, {
				props: {
					title: 'Panel',
					onDismiss: vi.fn(),
					dismissLabel: 'Close',
					initialFocus,
					children: body
				}
			});
			expect(view.getByRole('button', { name: 'Close' })).toHaveFocus();
			view.unmount();
		}
	});
});
