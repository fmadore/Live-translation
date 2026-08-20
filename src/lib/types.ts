// Shared types between the operator window, the caption overlay, and the Rust core.
// These mirror the serde structs in `src-tauri/src/types.rs` — keep them in sync.

/** Which audio input(s) to translate or transcribe. */
export type AudioSource = 'microphone' | 'system' | 'both';

/** A single capture source — the `origin` on captions, levels, and status updates. */
export type Origin = 'microphone' | 'system';

/** BCP-47 codes we use for the two caption languages. The spoken language is auto-detected. */
export type TargetLanguage = 'en' | 'fr';

/** Caption backend. The cloud providers each have their own API and API key; `ondevice`
 *  runs a local recognizer and needs no credential at all. Mirrors `Provider` in types.rs. */
export type Provider = 'gemini' | 'openai' | 'mistral' | 'ondevice';

/** Backends that produce translated captions. On-device recognition is same-language only:
 *  Windows exposes no on-device translation API. */
export function providerCanTranslate(provider: Provider): boolean {
	return provider === 'gemini' || provider === 'openai';
}

/** Whether an API key must be saved before a session can start. The on-device engine is the
 *  one backend that works with no credential, which is what keeps provider keys out of the
 *  app's primary functionality — see `docs/microsoft-store.md`. */
export function providerRequiresKey(provider: Provider): boolean {
	return provider !== 'ondevice';
}

/** Translate speech, or show a same-language transcription as live subtitles. */
export type OutputMode = 'translate' | 'transcribe';

export interface StartOptions {
	source: AudioSource;
	mode: OutputMode;
	/** Caption language (translation target). */
	targetLanguage: TargetLanguage;
	/** Realtime translation/transcription backend. */
	provider: Provider;
	/** Input device name for the microphone; null = system default. */
	micDeviceName?: string | null;
}

/** A caption update streamed from the active translation or subtitle session. */
export interface Caption {
	/** Monotonic id for the current utterance of this origin; a new turn increments it.
	 *  Only unique per origin — always key captions by (origin, turnId). */
	turnId: number;
	/** Text shown to the audience: translated text or same-language subtitles. */
	text: string;
	/** The recognised source-language text, for the operator monitor only. */
	sourceText: string;
	/** True once the turn is complete; until then it's an interim caption. */
	final: boolean;
	/** Which source produced it, when running both streams. */
	origin: Origin;
}

/** RMS level for the meter, 0.0–1.0, per source. */
export interface AudioLevel {
	source: Origin;
	rms: number;
	peak: number;
}

export type SessionState = 'idle' | 'connecting' | 'running' | 'reconnecting' | 'error';

export interface StatusUpdate {
	state: SessionState;
	/** Human-readable detail for the operator (e.g. reconnect reason). */
	message?: string;
	/** Which source this update is about; absent means the whole session (e.g. stop). */
	origin?: Origin;
}

export interface AudioDevice {
	name: string;
	isDefault: boolean;
}

/** Live overlay appearance/behaviour, pushed from the operator window to the overlay. */
export interface OverlayConfig {
	fontSize: number;
	/** Move mode: click-through is off and the overlay shows a drag region. */
	interactive?: boolean;
}

/** A finalized transcript line, kept for the on-screen log and disk export. */
export interface TranscriptLine {
	/** Monotonic id, unique for the app lifetime (stable list key) and the log's ordering. */
	id: number;
	text: string;
	sourceText: string;
	origin: Origin;
}

/** State the overlay reports back to the operator window, so the two stay in sync when the
 *  operator acts on the overlay itself rather than on the control panel. */
export interface OverlayStateMsg {
	/** False when the overlay leaves move mode ("Lock into place"). */
	interactive?: boolean;
	/** True once the caption region has been positioned on the presentation display. */
	placed?: boolean;
	/** Font size chosen from the overlay's own toolbar. */
	fontSize?: number;
}

/** Event names. Rust→front-end: caption/level/status. Operator→overlay: overlayConfig.
 *  Overlay→operator: overlayState. */
export const EVT = {
	caption: 'caption',
	level: 'audio-level',
	status: 'status',
	overlayConfig: 'overlay-config',
	overlayState: 'overlay-state',
} as const;

/** localStorage key shared by both windows (same origin) for the overlay font size. */
export const OVERLAY_FONT_KEY = 'overlay.fontSize';
export const DEFAULT_OVERLAY_FONT = 38;
export const OVERLAY_FONT_MIN = 20;
export const OVERLAY_FONT_MAX = 96;

/** Clamp a requested overlay font size to the supported range. */
export function clampOverlayFont(size: number): number {
	return Math.max(OVERLAY_FONT_MIN, Math.min(OVERLAY_FONT_MAX, Math.round(size)));
}

/** Read the persisted overlay font size (shared by both windows via localStorage). */
export function loadOverlayFont(): number {
	if (typeof localStorage === 'undefined') return DEFAULT_OVERLAY_FONT;
	const v = Number(localStorage.getItem(OVERLAY_FONT_KEY));
	return Number.isFinite(v) && v > 0 ? clampOverlayFont(v) : DEFAULT_OVERLAY_FONT;
}

/** Whether the caption region has ever been placed, so the pre-flight check survives a
 *  restart instead of asking the operator to position the overlay again. */
export const OVERLAY_PLACED_KEY = 'overlay.placed';

export function loadOverlayPlaced(): boolean {
	if (typeof localStorage === 'undefined') return false;
	return localStorage.getItem(OVERLAY_PLACED_KEY) === 'true';
}
