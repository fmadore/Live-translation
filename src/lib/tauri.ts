// Thin wrapper around the Tauri command/event API so the Svelte components stay clean,
// and so `npm run dev` in a plain browser (no Tauri runtime) degrades gracefully instead
// of throwing. When `window.__TAURI_INTERNALS__` is absent we are running in a browser.

import type {
	AudioDevice,
	Caption,
	AudioLevel,
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

// ---- Commands -------------------------------------------------------------

export const api = {
	listMicrophones: () => invoke<AudioDevice[]>('list_microphones'),

	hasApiKey: () => invoke<boolean>('has_api_key'),
	setApiKey: (key: string) => invoke<void>('set_api_key', { key }),
	clearApiKey: () => invoke<void>('clear_api_key'),

	startSession: (options: StartOptions) => invoke<void>('start_session', { options }),
	stopSession: () => invoke<void>('stop_session'),

	setOverlayClickThrough: (enabled: boolean) =>
		invoke<void>('set_overlay_click_through', { enabled }),
	showOverlay: (visible: boolean) => invoke<void>('show_overlay', { visible })
};

// ---- Events ---------------------------------------------------------------

export const on = {
	caption: (h: (c: Caption) => void) => listen<Caption>(EVT.caption, h),
	level: (h: (l: AudioLevel) => void) => listen<AudioLevel>(EVT.level, h),
	status: (h: (s: StatusUpdate) => void) => listen<StatusUpdate>(EVT.status, h)
};
