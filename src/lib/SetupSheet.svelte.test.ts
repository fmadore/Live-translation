import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render } from '@testing-library/svelte';
import { get } from 'svelte/store';
import SetupSheet from './SetupSheet.svelte';
import { createSetupActions } from './setupActions';
import { options } from './stores';
import { DEFAULT_START_OPTIONS } from './types';
import { setLocale } from './i18n';
import type { PreflightController } from './preflightController.svelte';

/** Just the preflight surface the sheet reads with the demo selected. */
function preflight(whisperRefused: boolean) {
	return {
		microphones: [],
		outputs: [],
		applications: [],
		applicationCaptureSupported: null,
		refreshing: false,
		refreshingApplications: false,
		refresh: vi.fn(),
		refreshApplications: vi.fn(),
		invalidateAudioTest: vi.fn(),
		whisperRefused,
		whisperCpuMissing: whisperRefused ? 'AVX, AVX2, FMA…' : ''
	} as unknown as PreflightController;
}

function mount(whisperRefused: boolean) {
	const actions = createSetupActions({
		locked: () => false,
		invalidateAudioTest: vi.fn(),
		refreshDevices: vi.fn(),
		whisperRefused: () => whisperRefused
	});
	return render(SetupSheet, {
		props: {
			actions,
			preflight: preflight(whisperRefused),
			locked: false,
			// Keeps the Whisper model panel from asking a core that is not there.
			browserMode: true,
			usesMic: true,
			usesSystem: false,
			languageError: ''
		}
	});
}

beforeEach(() => {
	setLocale('en');
	options.set({ ...DEFAULT_START_OPTIONS, provider: 'ondevice' });
});

describe('the engine step', () => {
	it('shows Whisper as unavailable, with the reason, on a processor that cannot run it', async () => {
		const view = mount(true);
		const whisper = view.getByRole('button', { name: /Local Whisper/ });
		expect(whisper).toBeDisabled();
		expect(whisper).toHaveTextContent(
			'Not available on this processor (it lacks AVX, AVX2, FMA…). The other engines still work.'
		);
		await fireEvent.click(whisper);
		expect(get(options).provider).toBe('ondevice');
		expect(view.getByRole('button', { name: /Built-in demo/ })).toHaveAttribute(
			'aria-pressed',
			'true'
		);
	});

	it('offers Whisper as usual when the processor can run it', async () => {
		const view = mount(false);
		const whisper = view.getByRole('button', { name: /Local Whisper/ });
		expect(whisper).toBeEnabled();
		expect(whisper).not.toHaveTextContent('Not available');
		await fireEvent.click(whisper);
		expect(get(options).provider).toBe('whisper');
	});
});
