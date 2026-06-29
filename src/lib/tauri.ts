// Thin wrapper around the Tauri command/event API so the Svelte components stay clean,
// and so `npm run dev` in a plain browser (no Tauri runtime) degrades gracefully instead
// of throwing. When `window.__TAURI_INTERNALS__` is absent we are running in a browser.

import type {
	AudioDevice,
	Caption,
	AudioLevel,
	OverlayConfig,
	Provider,
	StartOptions,
	StatusUpdate
} from './types';
import { EVT } from './types';

export const isTauri = (): boolean =>
	typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

type UnlistenFn = () => void;

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
	if (!isTauri()) {
		throw new Error(`Tauri command "${cmd}" called outside the desktop app`);
	}
	const { invoke } = await import('@tauri-apps/api/core');
	return invoke<T>(cmd, args);
}

async function listen<T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
	if (!isTauri()) return () => {};
	const { listen } = await import('@tauri-apps/api/event');
	return listen<T>(event, (e) => handler(e.payload));
}

async function emit<T>(event: string, payload: T): Promise<void> {
	if (!isTauri()) return;
	const { emit } = await import('@tauri-apps/api/event');
	await emit(event, payload);
}

// ---- Commands -------------------------------------------------------------

export const api = {
	listMicrophones: () => invoke<AudioDevice[]>('list_microphones'),

	hasApiKey: (provider: Provider) => invoke<boolean>('has_api_key', { provider }),
	setApiKey: (provider: Provider, key: string) =>
		invoke<void>('set_api_key', { provider, key }),
	clearApiKey: (provider: Provider) => invoke<void>('clear_api_key', { provider }),

	startSession: (options: StartOptions) => invoke<void>('start_session', { options }),
	stopSession: () => invoke<void>('stop_session'),

	setOverlayClickThrough: (enabled: boolean) =>
		invoke<void>('set_overlay_click_through', { enabled }),
	showOverlay: (visible: boolean) => invoke<void>('show_overlay', { visible }),

	/** Push live overlay appearance (font size) to the overlay window. */
	setOverlayConfig: (config: OverlayConfig) => emit(EVT.overlayConfig, config),

	/** Write the transcript to disk; returns the saved file path. */
	saveTranscript: (content: string, filename: string) =>
		invoke<string>('save_transcript', { content, filename })
};

// ---- Events ---------------------------------------------------------------

export const on = {
	caption: (h: (c: Caption) => void) => listen<Caption>(EVT.caption, h),
	level: (h: (l: AudioLevel) => void) => listen<AudioLevel>(EVT.level, h),
	status: (h: (s: StatusUpdate) => void) => listen<StatusUpdate>(EVT.status, h),
	overlayConfig: (h: (c: OverlayConfig) => void) => listen<OverlayConfig>(EVT.overlayConfig, h)
};
