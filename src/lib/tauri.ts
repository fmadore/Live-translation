// Thin wrapper around the Tauri command/event API so the Svelte components stay clean,
// and so `npm run dev` in a plain browser (no Tauri runtime) degrades gracefully instead
// of throwing. When `window.__TAURI_INTERNALS__` is absent we are running in a browser.

import type {
	AudioDevice,
	AudioSource,
	AudioTestUpdate,
	Caption,
	AudioLevel,
	OverlayConfig,
	OverlayStateMsg,
	OnDeviceReadiness,
	Provider,
	StartOptions,
	StatusUpdate,
	StoredRecovery
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
	onDeviceReadiness: () => invoke<OnDeviceReadiness>('ondevice_readiness'),
	prepareOnDeviceModel: () => invoke<OnDeviceReadiness>('prepare_ondevice_model'),

	startSession: (options: StartOptions) => invoke<void>('start_session', { options }),
	stopSession: () => invoke<void>('stop_session'),

	/** Level-only capture for the preflight: no provider connection, no captions, no stored
	 *  audio. See `SessionManager::start_test`. */
	startAudioTest: (source: AudioSource, micDeviceName: string | null) =>
		invoke<void>('start_audio_test', { source, micDeviceName }),
	stopAudioTest: () => invoke<void>('stop_audio_test'),

	setOverlayClickThrough: (enabled: boolean) =>
		invoke<void>('set_overlay_click_through', { enabled }),
	showOverlay: (visible: boolean) => invoke<void>('show_overlay', { visible }),

	/** Push live overlay appearance (font size) to the overlay window. */
	setOverlayConfig: (config: OverlayConfig) => emit(EVT.overlayConfig, config),

	/** Report a change made on the overlay itself back to the operator window. */
	emitOverlayState: (msg: OverlayStateMsg) => emit(EVT.overlayState, msg),

	/** Write the transcript to disk; returns the saved file path. */
	saveTranscript: (content: string, filename: string) =>
		invoke<string>('save_transcript', { content, filename }),

	/** Overwrite the crash-recovery spool; returns its path. Opt-in — see `recoveryEnabled`. */
	writeRecovery: (contents: string) => invoke<string>('write_recovery', { contents }),

	/** Read the spool left by a previous run, or null when there is nothing to recover. */
	readRecovery: () => invoke<StoredRecovery | null>('read_recovery'),

	/** Delete the spool. Called on save, clear, discard, and when recovery is switched off. */
	clearRecovery: () => invoke<void>('clear_recovery'),

	/** Tell the core whether closing the window would lose something, so it knows when to
	 *  intercept a close and when to leave it alone. See `shouldGuardClose`. */
	setCloseGuard: (guard: boolean) => invoke<void>('set_close_guard', { guard }),

	/** Say that an intercepted close is being handled. Sent immediately, before the session is
	 *  stopped: without it the core releases the window after `ACK_TIMEOUT` rather than let a
	 *  wedged renderer hold it shut. */
	ackClose: () => invoke<void>('ack_close'),

	/** Answer an intercepted close: quit for real. */
	confirmClose: () => invoke<void>('confirm_close')
};

// ---- Events ---------------------------------------------------------------

export const on = {
	caption: (h: (c: Caption) => void) => listen<Caption>(EVT.caption, h),
	level: (h: (l: AudioLevel) => void) => listen<AudioLevel>(EVT.level, h),
	status: (h: (s: StatusUpdate) => void) => listen<StatusUpdate>(EVT.status, h),
	audioTest: (h: (t: AudioTestUpdate) => void) => listen<AudioTestUpdate>(EVT.audioTest, h),
	/** The operator tried to close the window and the core held it open for an answer. */
	closeRequested: (h: () => void) => listen<null>(EVT.closeRequested, () => h()),
	overlayConfig: (h: (c: OverlayConfig) => void) => listen<OverlayConfig>(EVT.overlayConfig, h),
	overlayState: (h: (m: OverlayStateMsg) => void) => listen<OverlayStateMsg>(EVT.overlayState, h)
};
