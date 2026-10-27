import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, waitFor } from '@testing-library/svelte';
import { createRawSnippet, tick } from 'svelte';
import SessionControls from './SessionControls.svelte';
import OperatorToolbar from './OperatorToolbar.svelte';
import { setLocale } from './i18n';
import { originStates } from './stores';
import type { ZoomState } from './types';

/** Idle and ready to start. */
const idle = {
	busy: false,
	startDisabled: false,
	rehearseDisabled: false,
	paused: false,
	onStart: vi.fn(),
	onRehearse: vi.fn(),
	onStop: vi.fn(),
	onPause: vi.fn()
};

function controls(paused = false, onPause = vi.fn()) {
	return render(SessionControls, {
		props: {
			busy: false,
			startDisabled: false,
			rehearseDisabled: false,
			paused,
			onStart: vi.fn(),
			onRehearse: vi.fn(),
			onStop: vi.fn(),
			onPause
		}
	});
}

afterEach(() => {
	originStates.set({});
	setLocale('en');
});

describe('the session actions', () => {
	// The key used to float after Rehearse as a chip that read as a third button. It is part of
	// the action it triggers now, printed in the interface language and announced as a
	// shortcut rather than as part of the button's name.
	it('carry the start key on Start, outside its accessible name', () => {
		const start = controls().getAllByRole('button')[0];
		expect(start).toHaveAttribute('aria-keyshortcuts', 'Control+Shift+Space');
		expect(start).toHaveTextContent('Ctrl+Shift+Space');
		expect(start).not.toHaveAccessibleName(/Ctrl/);
	});

	it('print the key the way the interface language names it', () => {
		setLocale('de');
		expect(controls().getAllByRole('button')[0]).toHaveTextContent('Strg+Umschalt+Leertaste');
	});

	it('put the same key on Stop, because it stops too', () => {
		originStates.set({ system: 'running' });
		const stop = controls().getByRole('button', { name: /Stop/ });
		expect(stop).toHaveAttribute('aria-keyshortcuts', 'Control+Shift+Space');
		expect(stop).not.toHaveAccessibleName(/Ctrl/);
	});

	// The label says what a press does. A pressed state on top of it made Narrator read
	// "Resume, toggle button, pressed", which says the opposite of what is going on.
	it('offer Pause beside Stop while running, and Resume once paused, as plain actions', async () => {
		originStates.set({ system: 'running' });
		const onPause = vi.fn();
		const pause = controls(false, onPause).getByRole('button', { name: /Pause/ });
		expect(pause).not.toHaveAttribute('aria-pressed');
		expect(pause).toHaveAttribute('aria-keyshortcuts', 'Control+Shift+P');
		pause.click();
		expect(onPause).toHaveBeenCalledOnce();

		originStates.set({ system: 'paused' });
		const resume = controls(true).getByRole('button', { name: /Resume/ });
		expect(resume).not.toHaveAttribute('aria-pressed');
	});

	it('leave Rehearse without a shortcut of its own', () => {
		const rehearse = controls().getAllByRole('button')[1];
		expect(rehearse).not.toHaveAttribute('aria-keyshortcuts');
	});

	// Review U4: a disabled button drops the focus it holds, so the one pressed stays focusable
	// while it is busy and ignores presses instead.
	it('stay focusable while busy, and ignore a press', async () => {
		const onStart = vi.fn();
		const onRehearse = vi.fn();
		const view = render(SessionControls, {
			props: {
				...idle,
				busy: true,
				startDisabled: true,
				rehearseDisabled: true,
				onStart,
				onRehearse
			}
		});
		const [start, rehearse] = view.getAllByRole('button');
		for (const button of [start, rehearse]) {
			expect(button).not.toBeDisabled();
			expect(button).toHaveAttribute('aria-disabled', 'true');
			expect(button).toHaveAttribute('aria-busy', 'true');
			expect(button).toHaveAccessibleName('Starting…');
			await fireEvent.click(button);
		}
		expect(onStart).not.toHaveBeenCalled();
		expect(onRehearse).not.toHaveBeenCalled();
	});

	it('are disabled outright for anything else that blocks a start', () => {
		const view = render(SessionControls, {
			props: { ...idle, startDisabled: true, rehearseDisabled: true }
		});
		for (const button of view.getAllByRole('button')) {
			expect(button).toBeDisabled();
			expect(button).not.toHaveAttribute('aria-disabled');
		}
	});

	it('hand the keyboard from Start to Stop as the run begins, and back as it ends', async () => {
		const view = render(SessionControls, { props: { ...idle, busy: true } });
		// Start, pressed: Rehearse says "Starting…" as well.
		const [start] = view.getAllByRole('button', { name: 'Starting…' });
		start.focus();

		originStates.set({ microphone: 'connecting' });
		await waitFor(() => expect(view.getByRole('button', { name: 'Stopping…' })).toHaveFocus());
		await view.rerender({ busy: false });
		const stop = view.getByRole('button', { name: /Stop captions/ });
		expect(stop).toHaveFocus();

		originStates.set({ microphone: 'error' });
		await waitFor(() =>
			expect(view.getByRole('button', { name: /Start subtitles/ })).toHaveFocus()
		);
	});

	// A run that ends by itself while the operator is in the setup or the transcript leaves the
	// keyboard where it is.
	it('leave focus alone when it was somewhere else', async () => {
		const elsewhere = document.createElement('input');
		document.body.append(elsewhere);
		try {
			originStates.set({ microphone: 'running' });
			const view = render(SessionControls, { props: idle });
			elsewhere.focus();
			originStates.set({ microphone: 'error' });
			await view.findByRole('button', { name: /Start subtitles/ });
			await tick();
			expect(elsewhere).toHaveFocus();
		} finally {
			elsewhere.remove();
		}
	});
});

describe('the operator toolbar', () => {
	function toolbar(zoomState: ZoomState = 'off', onOpenZoom = vi.fn()) {
		return render(OperatorToolbar, {
			props: {
				elapsed: '00:00',
				settingsOpen: false,
				onOpenSettings: vi.fn(),
				zoomState,
				zoomOpen: false,
				onOpenZoom,
				actions: createRawSnippet(() => ({ render: () => '<button>Start</button>' }))
			}
		});
	}

	// Windows prints the name in the frame, so the bar no longer does; the heading that the
	// rest of the window hangs from has to survive that for Narrator's heading navigation.
	it('keeps the window a single h1, and renders the actions it is given', () => {
		const view = toolbar();
		expect(view.getByRole('heading', { level: 1 })).toHaveTextContent(
			'Live Translation & Subtitles'
		);
		expect(view.getByRole('button', { name: 'Start' })).toBeInTheDocument();
		expect(view.getByRole('button', { name: 'Open settings' })).toHaveAttribute(
			'aria-haspopup',
			'dialog'
		);
	});

	// Sharing with Zoom outlives the session, so its state has a place of its own in the bar:
	// named in full for a screen reader, and written beside the dot for anyone who cannot tell
	// its colours apart.
	it.each([
		['off', 'Zoom captions', ''],
		['sharing', 'Zoom captions: sharing', 'sharing'],
		['waiting', 'Zoom captions: waiting', 'waiting'],
		['refused', 'Zoom captions: refused', 'refused'],
		['unreachable', 'Zoom captions: unreachable', 'unreachable']
	] as const)('names the Zoom button for its state while %s', (state, name, word) => {
		const button = toolbar(state).getByRole('button', { name });
		expect(button).toHaveAttribute('aria-haspopup', 'dialog');
		expect(button).toHaveAttribute('aria-expanded', 'false');
		expect(button.textContent?.replace(/\s+/g, ' ').trim()).toBe(`Zoom captions ${word}`.trim());
	});

	it('opens the Zoom dialog from its button', async () => {
		const onOpenZoom = vi.fn();
		await fireEvent.click(
			toolbar('off', onOpenZoom).getByRole('button', { name: 'Zoom captions' })
		);
		expect(onOpenZoom).toHaveBeenCalledOnce();
	});
});
