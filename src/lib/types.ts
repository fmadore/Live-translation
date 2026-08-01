// Shared types between the operator window, the caption overlay, and the Rust core.
// These mirror the serde structs in `src-tauri/src/types.rs` — keep them in sync.

/** Which audio input(s) to translate or transcribe. */
export type AudioSource = 'microphone' | 'system' | 'both';

/** A single capture source — the `origin` on captions, levels, and status updates. */
export type Origin = 'microphone' | 'system';

/** BCP-47 codes we use for the two workshop languages. The spoken language is auto-detected. */
export type TargetLanguage = 'en' | 'fr';

/** Realtime provider / backend. Each has its own API and API key. */
export type Provider = 'gemini' | 'openai' | 'mistral';

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
	/** Monotonic id, unique for the app lifetime (stable list key). */
	id: number;
	/** Local clock time the line was finalized, e.g. "14:03:21". */
	time: string;
	text: string;
	sourceText: string;
	origin: Origin;
}

/** Event names. Rust→front-end: caption/level/status. Operator→overlay: overlayConfig. */
export const EVT = {
	caption: 'caption',
	level: 'audio-level',
	status: 'status',
	overlayConfig: 'overlay-config',
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
