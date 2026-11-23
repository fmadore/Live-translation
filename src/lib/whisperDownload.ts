// The local Whisper models on this PC and the one download the core runs at a time. Shared by
// the model step of the setup and the pre-flight checklist's Whisper row, so a download started
// from either shows in both and neither can start a second one.

import { derived, get, writable } from 'svelte/store';
import { api } from './tauri';
import { asStatus } from './errors';
import { statusMessage, whisperModels } from './stores';
import type { WhisperModelId } from './types';

/** A download or removal this window asked for has not returned yet. */
export const whisperModelBusy = writable(false);

/** The core reports a download under way, whichever window started it. */
export const whisperDownloading = derived(whisperModels, (models) =>
	models.some((m) => m.downloading)
);

let refreshing: Promise<void> | null = null;
let refreshAgain = false;

/** Ask the core which models are installed, and how far a download has got. A call while a
 *  read is under way reads once more after it, and resolves with that read: the answer in
 *  flight may predate what the caller just did, such as the end of a download or a removal,
 *  and the poll that stops then would leave the list as it was before. */
function refreshWhisperModels(): Promise<void> {
	if (refreshing) {
		refreshAgain = true;
		return refreshing;
	}
	refreshing = (async () => {
		try {
			do {
				refreshAgain = false;
				whisperModels.set(await api.whisperModels());
			} while (refreshAgain);
		} catch (error) {
			statusMessage.set(asStatus(error));
		} finally {
			refreshing = null;
		}
	})();
	return refreshing;
}

/** One change to the installed models at a time, never during a download, and the list
 *  re-read afterwards whether it worked or not. */
async function change(work: () => Promise<void>) {
	if (get(whisperModelBusy) || get(whisperDownloading)) return;
	whisperModelBusy.set(true);
	try {
		await work();
	} catch (error) {
		statusMessage.set(asStatus(error));
	} finally {
		whisperModelBusy.set(false);
		await refreshWhisperModels();
	}
}

export function downloadWhisperModel(id: WhisperModelId) {
	return change(async () => {
		statusMessage.set('');
		await api.downloadWhisperModel(id);
	});
}

export function removeWhisperModel(id: WhisperModelId) {
	return change(() => api.removeWhisperModel(id));
}

export async function cancelWhisperDownload() {
	try {
		await api.cancelWhisperDownload();
	} catch (error) {
		statusMessage.set(asStatus(error));
	}
}

let watchers = 0;
let timer: ReturnType<typeof setInterval> | undefined;

/** Read the models now, then every 500 ms while a download or removal is under way, for as
 *  long as anything showing them is mounted. Returns the function that stops watching. */
export function watchWhisperModels(): () => void {
	void refreshWhisperModels();
	if (watchers++ === 0) {
		timer = setInterval(() => {
			if (get(whisperModelBusy) || get(whisperDownloading)) void refreshWhisperModels();
		}, 500);
	}
	let stopped = false;
	return () => {
		if (stopped) return;
		stopped = true;
		if (--watchers === 0) clearInterval(timer);
	};
}
