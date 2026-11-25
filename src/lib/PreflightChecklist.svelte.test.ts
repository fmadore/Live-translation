import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, waitFor, within } from '@testing-library/svelte';
import PreflightChecklist from './PreflightChecklist.svelte';
import { forgetKnownKeys } from './ApiKeyPanel.svelte';
import { operatorContextMap } from './operatorContext';
import { options, whisperModels } from './stores';
import { DEFAULT_START_OPTIONS } from './startOptions';
import { get } from 'svelte/store';
import { setLocale } from './i18n';
import type { OverlayController } from './overlayController.svelte';
import type { PreflightController } from './preflightController.svelte';
import type { StartOptions, WhisperModelInfo } from './types';

const mocks = vi.hoisted(() => ({
	models: vi.fn(),
	download: vi.fn(),
	cancel: vi.fn(),
	hasApiKey: vi.fn()
}));
vi.mock('./tauri', () => ({
	isTauri: () => true,
	api: {
		whisperModels: mocks.models,
		downloadWhisperModel: mocks.download,
		cancelWhisperDownload: mocks.cancel,
		hasApiKey: mocks.hasApiKey
	}
}));

/** Just the pre-flight surface the checklist reads. */
const preflight = {
	localReadiness: null,
	micVerified: false,
	systemVerified: false,
	micSignal: false,
	systemSignal: false,
	applicationCaptureSupported: null,
	applicationReady: () => true,
	audioTesting: false,
	audioTestBusy: false,
	systemSilenced: false,
	whisperRefused: false,
	whisperCpuMissing: '',
	noteKey: vi.fn(),
	startAudioTest: vi.fn(),
	stopAudioTest: vi.fn()
} as unknown as PreflightController;
const overlay = { moveOverlay: false } as unknown as OverlayController;

const BASE: WhisperModelInfo = {
	id: 'base',
	bytes: 59707625,
	installed: false,
	downloading: false,
	downloadedBytes: 0,
	inUse: false
};

function mount(setup: Partial<StartOptions>, usesMic = true, usesSystem = false) {
	options.set({ ...DEFAULT_START_OPTIONS, ...setup });
	return render(PreflightChecklist, {
		props: { browserMode: false, locked: false, usesMic, usesSystem },
		context: operatorContextMap({ preflight, overlay })
	});
}

beforeEach(() => {
	setLocale('en');
	forgetKnownKeys();
	whisperModels.set([]);
	mocks.models.mockResolvedValue([{ ...BASE }]);
	mocks.hasApiKey.mockResolvedValue(true);
});

/** The checklist row whose title is `title`. */
function row(view: ReturnType<typeof mount>, title: string): HTMLElement {
	return view.getByText(title, { selector: '.check-title' }).closest('.check-row') as HTMLElement;
}

describe('the running cost row', () => {
	const translating: Partial<StartOptions> = {
		mode: 'translate',
		provider: 'gemini',
		targetLanguage: 'en',
		secondTargetLanguage: 'fr',
		source: 'both'
	};

	it('multiplies the per-stream rate by both sources and both languages, as the rail does', () => {
		const view = mount(translating, true, true);
		const cost = row(view, 'Running cost');
		expect(cost).toHaveTextContent('×2 sources');
		expect(cost).toHaveTextContent('×2 languages');
	});

	// D2: the rate and its tags were loose siblings in a three-column grid, so a tag became a
	// fourth grid item and wrapped into the 24px mark column.
	it('keeps the rate and its tags in one grid cell', () => {
		const view = mount(translating, true, true);
		const cost = row(view, 'Running cost');
		expect(cost.children).toHaveLength(3);
		const action = cost.children[2];
		expect(within(action as HTMLElement).getByText('×2 sources')).toBeTruthy();
		expect(within(action as HTMLElement).getByText('×2 languages')).toBeTruthy();
	});

	it('names neither multiplier for one source in one language', () => {
		const view = mount({ ...translating, secondTargetLanguage: null }, true, false);
		const cost = row(view, 'Running cost');
		expect(cost).not.toHaveTextContent('×2');
	});

	it('names no second source for local Whisper, which bills nothing per stream', () => {
		const view = mount({ mode: 'transcribe', provider: 'whisper', source: 'both' }, true, true);
		expect(row(view, 'Running cost')).not.toHaveTextContent('×2 sources');
	});
});

describe('the loudspeaker reminder', () => {
	const both: Partial<StartOptions> = { mode: 'transcribe', provider: 'whisper', source: 'both' };

	it('reminds the operator under Both sources, and turns the echo filter on from the row', async () => {
		const view = mount(both, true, true);
		const reminder = row(view, 'Loudspeakers?');
		expect(reminder).toHaveTextContent('captioned twice');
		const toggle = within(reminder).getByRole('button', { name: 'Leave out the call' });
		expect(toggle).toHaveAttribute('aria-pressed', 'false');

		await fireEvent.click(toggle);
		expect(get(options).dropMicEcho).toBe(true);
		expect(toggle).toHaveAttribute('aria-pressed', 'true');
		expect(reminder).toHaveTextContent('will be left out');

		await fireEvent.click(toggle);
		expect(get(options).dropMicEcho).toBe(false);
	});

	it('is not shown for one source or for the built-in demo', () => {
		expect(mount({ ...both, source: 'microphone' }).queryByText('Loudspeakers?')).toBeNull();
		expect(
			mount({ ...both, provider: 'ondevice' }, true, true).queryByText('Loudspeakers?')
		).toBeNull();
	});

	it('cannot be changed while setup is locked', () => {
		options.set({ ...DEFAULT_START_OPTIONS, ...both });
		const view = render(PreflightChecklist, {
			props: { browserMode: false, locked: true, usesMic: true, usesSystem: true },
			context: operatorContextMap({ preflight, overlay })
		});
		expect(
			within(row(view, 'Loudspeakers?')).getByRole('button', { name: 'Leave out the call' })
		).toBeDisabled();
	});
});

describe('the Whisper row on a first run', () => {
	it('offers the download of the selected model, with its size, and starts it', async () => {
		let finish!: () => void;
		mocks.download.mockReturnValue(new Promise<void>((resolve) => (finish = resolve)));
		const view = mount({ mode: 'transcribe', provider: 'whisper' });
		const whisper = () => row(view, 'Local transcription');
		const download = await waitFor(() =>
			within(whisper()).getByRole('button', { name: /^Download Base \(57\sMiB\)$/ })
		);
		expect(download).toHaveAccessibleDescription('Download a model before starting.');
		download.focus();
		await fireEvent.click(download);
		expect(mocks.download).toHaveBeenCalledExactlyOnceWith('base');
		// The core reports the download; the row shows how far it has got and offers a way out.
		whisperModels.set([{ ...BASE, downloading: true, downloadedBytes: BASE.bytes / 2 }]);
		const bar = await waitFor(() => within(whisper()).getByRole('progressbar'));
		expect(bar).toHaveAttribute('aria-valuetext', '50%');
		expect(whisper()).toHaveTextContent('Downloading model…');
		const cancel = within(whisper()).getByRole('button', { name: 'Cancel download' });
		await fireEvent.click(cancel);
		expect(mocks.cancel).toHaveBeenCalledOnce();

		mocks.models.mockResolvedValue([{ ...BASE, installed: true }]);
		finish();
		await waitFor(() =>
			expect(whisper()).toHaveTextContent('Model ready for offline transcription.')
		);
		expect(within(whisper()).queryByRole('button')).toBeNull();
		// The button the keyboard was on is gone: focus lands on the row's sentence, not the page.
		await waitFor(() => expect(document.activeElement).toHaveClass('check-desc'));
		expect(document.activeElement).toHaveTextContent('Model ready');
	});

	it('hands the keyboard back to Download after a cancelled download', async () => {
		let finish!: () => void;
		mocks.download.mockReturnValue(new Promise<void>((resolve) => (finish = resolve)));
		const view = mount({ mode: 'transcribe', provider: 'whisper' });
		const download = await waitFor(() => view.getByRole('button', { name: /^Download Base/ }));
		download.focus();
		await fireEvent.click(download);
		whisperModels.set([{ ...BASE, downloading: true, downloadedBytes: 1 }]);
		const cancel = await waitFor(() => view.getByRole('button', { name: 'Cancel download' }));
		await waitFor(() => expect(cancel).toBeEnabled());
		cancel.focus();
		await fireEvent.click(cancel);
		finish();
		await waitFor(() =>
			expect(view.getByRole('button', { name: /^Download Base \(57\sMiB\)$/ })).toHaveFocus()
		);
	});

	it('names the model the operator chose', async () => {
		mocks.models.mockResolvedValue([{ ...BASE, id: 'small', bytes: 487_601_967 }]);
		const view = mount({ mode: 'transcribe', provider: 'whisper', whisperModel: 'small' });
		await waitFor(() =>
			expect(view.getByRole('button', { name: /^Download Small \(466\sMiB\)$/ })).toBeEnabled()
		);
	});

	it('offers nothing once the model is installed', async () => {
		mocks.models.mockResolvedValue([{ ...BASE, installed: true }]);
		const view = mount({ mode: 'transcribe', provider: 'whisper' });
		const whisper = row(view, 'Local transcription');
		await waitFor(() => expect(whisper).toHaveTextContent('Model ready'));
		expect(within(whisper).queryByRole('button')).toBeNull();
	});

	it('keeps the download closed while setup is locked', async () => {
		options.set({ ...DEFAULT_START_OPTIONS, mode: 'transcribe', provider: 'whisper' });
		const view = render(PreflightChecklist, {
			props: { browserMode: false, locked: true, usesMic: true, usesSystem: false },
			context: operatorContextMap({ preflight, overlay })
		});
		const download = await waitFor(() => view.getByRole('button', { name: /^Download Base/ }));
		expect(download).toBeDisabled();
	});
});
