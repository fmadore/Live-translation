import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import { api } from './tauri';
import { whisperModels } from './stores';
import { removeWhisperModel, watchWhisperModels } from './whisperDownload';
import type { WhisperModelInfo } from './types';

vi.mock('./tauri', () => ({
	api: {
		whisperModels: vi.fn(),
		removeWhisperModel: vi.fn(),
		downloadWhisperModel: vi.fn(),
		cancelWhisperDownload: vi.fn()
	}
}));

const base = (installed: boolean): WhisperModelInfo[] => [
	{ id: 'base', bytes: 100, installed, downloading: false, downloadedBytes: 0, inUse: false }
];

function deferred<T>() {
	let resolve!: (value: T) => void;
	const promise = new Promise<T>((done) => (resolve = done));
	return { promise, resolve };
}

beforeEach(() => {
	vi.useFakeTimers();
});
afterEach(() => {
	vi.useRealTimers();
	vi.mocked(api.whisperModels).mockReset();
	vi.mocked(api.removeWhisperModel).mockReset();
});

// #240: a removal that ended while a poll was reading the models left the poll's older answer
// on screen: the final refresh returned at once, and polling stopped with the removal.
it('ends on the fresh list when a removal ends during a poll', async () => {
	vi.mocked(api.whisperModels).mockResolvedValueOnce(base(true));
	const stop = watchWhisperModels();
	await vi.waitFor(() => expect(get(whisperModels)).toEqual(base(true)));

	const removal = deferred<void>();
	vi.mocked(api.removeWhisperModel).mockReturnValueOnce(removal.promise);
	const removing = removeWhisperModel('base');

	// The poll asks while the removal runs; its answer comes back only after the removal ended.
	const poll = deferred<WhisperModelInfo[]>();
	vi.mocked(api.whisperModels).mockReturnValueOnce(poll.promise).mockResolvedValue(base(false));
	await vi.advanceTimersByTimeAsync(500);
	expect(api.whisperModels).toHaveBeenCalledTimes(2);

	removal.resolve();
	await Promise.resolve();
	poll.resolve(base(true));
	await removing;
	expect(get(whisperModels)).toEqual(base(false));
	expect(api.whisperModels).toHaveBeenCalledTimes(3);

	// Nothing is under way any more, so the poll stays quiet.
	await vi.advanceTimersByTimeAsync(2000);
	expect(api.whisperModels).toHaveBeenCalledTimes(3);
	stop();
});
