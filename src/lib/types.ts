// Shared types between the operator window, the caption overlay, and the Rust core.
// These mirror the serde structs in `src-tauri/src/types.rs` — keep them in sync.

/** Which audio input(s) to translate. */
export type AudioSource = 'microphone' | 'system' | 'both';

/**
 * Translation direction. The Live Translate model auto-detects the spoken language;
 * `targetLanguageCode` is the language we want captions in.
 *
 * - `fr-to-en`  : target English (a French speaker → English captions)
 * - `en-to-fr`  : target French  (an English speaker → French captions)
 * - `auto`      : let the operator flip quickly; we still send a concrete target,
 *                 defaulting to English, and expose a one-key toggle in the UI.
 */
export type Direction = 'fr-to-en' | 'en-to-fr';

/** BCP-47 codes we use for the two workshop languages. */
export type TargetLanguage = 'en' | 'fr';

/**
 * How we turn speech into translated text:
 * - `live-translate` : dedicated speech-to-speech model; captions from its output
 *                      transcription sidecar (audio discarded). Purpose-tuned for translation.
 * - `speech-to-text` : general Live model with TEXT output + a translate prompt; audio in,
 *                      translated text out, no audio synthesized.
 */
export type TranslationMode = 'live-translate' | 'speech-to-text';

export interface StartOptions {
	source: AudioSource;
	/** Caption language (translation target). */
	targetLanguage: TargetLanguage;
	/** Translation engine / model path. */
	mode: TranslationMode;
	/** Input device name for the microphone; null = system default. */
	micDeviceName?: string | null;
}

/** A caption update streamed from the Gemini Live session. */
export interface Caption {
	/** Monotonic id for the current utterance; a new turn increments it. */
	turnId: number;
	/** The translated text shown to the audience (output transcription). */
	text: string;
	/** The recognised source-language text, for the operator monitor only. */
	sourceText: string;
	/** True once Gemini marks the turn complete; until then it's an interim caption. */
	final: boolean;
	/** Which source produced it, when running both streams. */
	origin: AudioSource;
}

/** RMS level for the meter, 0.0–1.0, per source. */
export interface AudioLevel {
	source: 'microphone' | 'system';
	rms: number;
	peak: number;
}

export type SessionState = 'idle' | 'connecting' | 'running' | 'reconnecting' | 'error';

export interface StatusUpdate {
	state: SessionState;
	/** Human-readable detail for the operator (e.g. reconnect reason). */
	message?: string;
}

export interface AudioDevice {
	name: string;
	isDefault: boolean;
}

/** Live overlay appearance, pushed from the operator window to the overlay window. */
export interface OverlayConfig {
	fontSize: number;
}

/** A finalized transcript line, kept for the on-screen log and disk export. */
export interface TranscriptLine {
	/** Local clock time the line was finalized, e.g. "14:03:21". */
	time: string;
	text: string;
	sourceText: string;
	origin: AudioSource;
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
