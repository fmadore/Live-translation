import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { render, waitFor } from '@testing-library/svelte';
import IdleStage from './IdleStage.svelte';
import LaunchNotes from './LaunchNotes.svelte';
import { operatorContextMap } from './operatorContext';
import { createOverlayController } from './overlayController.svelte';
import { options, recoveryEnabled, statusMessage } from './stores';
import { historyEnabled } from './history';
import { recoveryFailure } from './recovery';
import { DEFAULT_START_OPTIONS } from './startOptions';
import { setLocale } from './i18n';
import type { PreflightController } from './preflightController.svelte';

// S4: the idle stage around the saved transcript — the pre-flight head above it and the
// launch notes below — as the page lays them out.

/** Just the pre-flight surface the checklist reads with the demo selected. */
const preflight = {
	localReadiness: { ready: true, engine: 'built-in-demo', state: 'ready', canPrepare: false },
	micVerified: false,
	systemVerified: false,
	micSignal: false,
	systemSignal: false,
	audioTesting: false,
	audioTestBusy: false,
	applicationReady: () => true
} as unknown as PreflightController;

beforeEach(() => {
	setLocale('en');
	options.set({ ...DEFAULT_START_OPTIONS, provider: 'ondevice' });
	statusMessage.set('');
});
afterEach(() => {
	historyEnabled.set(false);
	recoveryEnabled.set(false);
	statusMessage.set('');
});

it('heads the idle stage with what to check, then the checklist', () => {
	const view = render(IdleStage, {
		props: { browserMode: false, locked: false, usesMic: true, usesSystem: false },
		context: operatorContextMap({ preflight, overlay: createOverlayController() })
	});
	expect(view.getByRole('heading', { level: 2, name: 'Ready when you are' })).toBeInTheDocument();
	expect(view.getByText('Overlay placement')).toBeInTheDocument();
	expect(view.getByText('Running cost')).toBeInTheDocument();
});

describe('the launch notes', () => {
	it('say where the transcript is kept: memory, the recovery spool or history', async () => {
		const view = render(LaunchNotes, { props: { statusText: '' } });
		expect(view.getByText(/^Transcript is held in memory until you save it\./)).toBeInTheDocument();
		recoveryEnabled.set(true);
		await waitFor(() =>
			expect(
				view.getByText(/^Transcript is held in memory and spooled locally/)
			).toBeInTheDocument()
		);
		// Review U3: a recovery copy that keeps failing is not described as being kept.
		recoveryFailure.set({ id: 'error.recoveryWrite', detail: 'disk full' });
		await waitFor(() =>
			expect(view.getByText(/Its local recovery copy is failing\./)).toBeInTheDocument()
		);
		recoveryFailure.set('');
		historyEnabled.set(true);
		await waitFor(() =>
			expect(view.getByText(/^Transcript history is saved locally/)).toBeInTheDocument()
		);
	});

	it('repeat the status line for the eye, hidden from the screen reader that hears it', () => {
		statusMessage.set('Disk full');
		const view = render(LaunchNotes, { props: { statusText: 'Disk full' } });
		expect(view.getByText('Disk full')).toHaveAttribute('aria-hidden', 'true');
	});
});
