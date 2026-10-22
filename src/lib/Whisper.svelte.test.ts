import { beforeEach, expect, it, vi } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import { get } from 'svelte/store';
import WhisperModels from './WhisperModels.svelte';
import WhisperActivity from './WhisperActivity.svelte';
import { options, whisperModels, statusMessage, originStates } from './stores';
import { DEFAULT_START_OPTIONS } from './startOptions';
import type { WhisperModelInfo, WhisperProgress } from './types';
import { setLocale } from './i18n';
import { whisperPendingSeconds, whisperProgress } from './whisperProgress';

const mocks = vi.hoisted(() => ({
	models: vi.fn(),
	download: vi.fn(),
	cancel: vi.fn(),
	remove: vi.fn(),
	discard: vi.fn(),
	progress: vi.fn(),
	unlisten: vi.fn()
}));
vi.mock('./tauri', () => ({
	api: {
		whisperModels: mocks.models,
		downloadWhisperModel: mocks.download,
		cancelWhisperDownload: mocks.cancel,
		removeWhisperModel: mocks.remove,
		discardWhisperPending: mocks.discard
	},
	on: { whisperProgress: mocks.progress }
}));
const base: WhisperModelInfo = {
	id: 'base',
	bytes: 59707625,
	installed: false,
	downloading: false,
	downloadedBytes: 0,
	inUse: false
};

beforeEach(() => {
	vi.resetAllMocks();
	setLocale('en');
	options.set({ ...DEFAULT_START_OPTIONS });
	whisperModels.set([]);
	statusMessage.set('');
	originStates.set({});
	whisperProgress.set({});
	mocks.models.mockResolvedValue([{ ...base }]);
	mocks.progress.mockResolvedValue(mocks.unlisten);
});

it('offers the first-launch model download without downloading automatically', async () => {
	const view = render(WhisperModels, { locked: false, browserMode: false });
	await waitFor(() => expect(mocks.models).toHaveBeenCalled());
	expect(view.getByRole('combobox', { name: 'Whisper model' })).toHaveValue('base');
	expect(view.getByRole('button', { name: 'Download model' })).toBeEnabled();
	expect(mocks.download).not.toHaveBeenCalled();
});

it('downloads the selected model, exposes cancellation, then refreshes offline readiness', async () => {
	let finish!: () => void;
	mocks.download.mockReturnValue(
		new Promise<void>((resolve) => {
			finish = resolve;
		})
	);
	const view = render(WhisperModels, { locked: false, browserMode: false });
	await waitFor(() => expect(mocks.models).toHaveBeenCalled());
	await fireEvent.click(view.getByRole('button', { name: 'Download model' }));
	expect(mocks.download).toHaveBeenCalledWith('base');
	expect(view.getByRole('combobox', { name: 'Whisper model' })).toBeDisabled();
	whisperModels.set([{ ...base, downloading: true, downloadedBytes: base.bytes / 2 }]);
	await waitFor(() =>
		expect(view.getByRole('progressbar')).toHaveAttribute('value', String(base.bytes / 2))
	);
	await fireEvent.click(view.getByRole('button', { name: 'Cancel download' }));
	expect(mocks.cancel).toHaveBeenCalledOnce();
	mocks.models.mockResolvedValue([{ ...base, installed: true }]);
	finish();
	await waitFor(() =>
		expect(view.getByText('Model ready for offline transcription.')).toBeInTheDocument()
	);
	expect(view.getByRole('button', { name: 'Remove model' })).toBeEnabled();
});

it('keeps a failed download retryable and surfaces its error', async () => {
	mocks.download.mockRejectedValue(new Error('offline download'));
	const view = render(WhisperModels, { locked: false, browserMode: false });
	await fireEvent.click(view.getByRole('button', { name: 'Download model' }));
	await waitFor(() => expect(JSON.stringify(get(statusMessage))).toContain('offline download'));
	expect(view.getByRole('button', { name: 'Download model' })).toBeEnabled();
	expect(get(whisperModels)[0].installed).toBe(false);
});

it('prevents model mutation in a running session and avoids native calls in a browser preview', async () => {
	const view = render(WhisperModels, { locked: true, browserMode: true });
	expect(view.getByRole('button', { name: 'Download model' })).toBeDisabled();
	expect(view.getByRole('combobox')).toBeDisabled();
	expect(mocks.models).not.toHaveBeenCalled();
	await view.rerender({ locked: false, browserMode: true });
	expect(view.getByRole('button', { name: 'Download model' })).toBeDisabled();
});

it('requires a separate confirmation to discard pending audio and clears progress when finished', async () => {
	let progress!: (p: WhisperProgress) => void;
	mocks.progress.mockImplementation((handler) => {
		progress = handler;
		return Promise.resolve(mocks.unlisten);
	});
	originStates.set({ system: 'running' });
	const view = render(WhisperActivity);
	await waitFor(() => expect(mocks.progress).toHaveBeenCalled());
	progress({ origin: 'system', pendingMs: 30000, finalizing: true });
	await waitFor(() => expect(view.getByText(/Finishing the transcript.*30 s/)).toBeInTheDocument());
	// Shared with the quit prompt, which says how much closing would discard. The longer
	// source is what finishing waits for.
	progress({ origin: 'microphone', pendingMs: 41500, finalizing: true });
	expect(get(whisperPendingSeconds)).toBe(42);
	await fireEvent.click(view.getByRole('button', { name: 'Discard remaining audio' }));
	expect(mocks.discard).not.toHaveBeenCalled();
	expect(view.getByRole('alert')).toHaveTextContent('transcript will be incomplete');
	await fireEvent.click(view.getByRole('button', { name: 'Cancel' }));
	expect(view.queryByRole('alert')).not.toBeInTheDocument();
	await fireEvent.click(view.getByRole('button', { name: 'Discard remaining audio' }));
	await fireEvent.click(view.getAllByRole('button', { name: 'Discard remaining audio' })[1]);
	expect(mocks.discard).toHaveBeenCalledOnce();
	originStates.set({ system: 'idle' });
	await waitFor(() => expect(view.queryByText(/Finishing the transcript/)).not.toBeInTheDocument());
	expect(get(whisperPendingSeconds)).toBe(0);
	view.unmount();
	expect(mocks.unlisten).toHaveBeenCalledOnce();
});

/** Counts what Narrator would hear: a live region is announced when its content changes. */
function watchLiveRegion(region: HTMLElement) {
	const observer = new MutationObserver(() => {});
	observer.observe(region, { childList: true, characterData: true, subtree: true });
	return () => {
		const changes = observer.takeRecords().length;
		observer.disconnect();
		return changes;
	};
}

// Review U2: the download is polled every 500 ms and its status said the percentage, so
// Narrator read "Downloading 41%", "42%"… for the whole download.
it('announces the download starting and the model ready, and leaves the percentage to the bar', async () => {
	let finish!: () => void;
	mocks.download.mockReturnValue(
		new Promise<void>((resolve) => {
			finish = resolve;
		})
	);
	const view = render(WhisperModels, { locked: false, browserMode: false });
	await waitFor(() => expect(view.getByRole('status')).toHaveTextContent('Download a model'));
	await fireEvent.click(view.getByRole('button', { name: 'Download model' }));
	whisperModels.set([{ ...base, downloading: true, downloadedBytes: base.bytes / 4 }]);
	const status = view.getByRole('status');
	await waitFor(() => expect(status).toHaveTextContent('Downloading model…'));
	expect(view.getByRole('progressbar', { name: 'Downloading model…' })).toHaveAttribute(
		'aria-valuetext',
		'25%'
	);

	const changes = watchLiveRegion(status);
	whisperModels.set([{ ...base, downloading: true, downloadedBytes: base.bytes / 2 }]);
	await waitFor(() =>
		expect(view.getByRole('progressbar')).toHaveAttribute('aria-valuetext', '50%')
	);
	expect(status).toHaveTextContent(/^Downloading model…$/);
	expect(changes()).toBe(0);

	mocks.models.mockResolvedValue([{ ...base, installed: true }]);
	finish();
	await waitFor(() => expect(status).toHaveTextContent('Model ready for offline transcription.'));
	expect(view.queryByRole('progressbar')).not.toBeInTheDocument();
});

// Review U2: progress arrives about once a second and the backlog includes the window still
// being filled, so a status around the seconds spoke all session. The strip now appears only
// once Whisper is behind, and nothing on it is live.
it('shows the backlog only once Whisper falls behind, without announcing the count', async () => {
	let progress!: (p: WhisperProgress) => void;
	mocks.progress.mockImplementation((handler) => {
		progress = handler;
		return Promise.resolve(mocks.unlisten);
	});
	originStates.set({ system: 'running' });
	const view = render(WhisperActivity);
	await waitFor(() => expect(mocks.progress).toHaveBeenCalled());
	const status = view.getByRole('status');
	const changes = watchLiveRegion(status);

	// A window filling while the one before it is transcribed: a session keeping up.
	progress({ origin: 'system', pendingMs: 20000, finalizing: false });
	await waitFor(() => expect(get(whisperPendingSeconds)).toBe(20));
	expect(view.queryByText(/Audio waiting to be processed/)).not.toBeInTheDocument();

	progress({ origin: 'system', pendingMs: 30000, finalizing: false });
	await waitFor(() =>
		expect(view.getByText('Audio waiting to be processed · 30 s')).toBeInTheDocument()
	);
	progress({ origin: 'system', pendingMs: 31000, finalizing: false });
	await waitFor(() =>
		expect(view.getByText('Audio waiting to be processed · 31 s')).toBeInTheDocument()
	);
	expect(view.getByText(/Audio waiting/).closest('[role]')).toBeNull();
	expect(status.textContent).toBe('');
	expect(changes()).toBe(0);
	view.unmount();
});

it('announces finishing once and a confirmed discard once, not the seconds counting down', async () => {
	let progress!: (p: WhisperProgress) => void;
	mocks.progress.mockImplementation((handler) => {
		progress = handler;
		return Promise.resolve(mocks.unlisten);
	});
	originStates.set({ system: 'running' });
	const view = render(WhisperActivity);
	await waitFor(() => expect(mocks.progress).toHaveBeenCalled());
	const status = view.getByRole('status');

	// Finishing is shown whatever the backlog, because Stop is waiting on it.
	progress({ origin: 'system', pendingMs: 8000, finalizing: true });
	await waitFor(() => expect(status).toHaveTextContent('Finishing the transcript…'));
	expect(view.getByText('Finishing the transcript… · 8 s')).toBeInTheDocument();
	const changes = watchLiveRegion(status);
	for (const pendingMs of [7000, 6000, 5000]) {
		progress({ origin: 'system', pendingMs, finalizing: true });
		await waitFor(() =>
			expect(view.getByText(`Finishing the transcript… · ${pendingMs / 1000} s`)).toBeTruthy()
		);
	}
	expect(changes()).toBe(0);
	expect(status).toHaveTextContent(/^Finishing the transcript…$/);

	await fireEvent.click(view.getByRole('button', { name: 'Discard remaining audio' }));
	await fireEvent.click(view.getAllByRole('button', { name: 'Discard remaining audio' })[1]);
	expect(mocks.discard).toHaveBeenCalledOnce();
	await waitFor(() => expect(status).toHaveTextContent('Discarding the remaining audio…'));

	// The session's own status says how it ended; this region falls silent.
	originStates.set({ system: 'idle' });
	await waitFor(() => expect(status.textContent).toBe(''));
	view.unmount();
});
