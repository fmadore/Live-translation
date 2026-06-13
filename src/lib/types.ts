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

export interface StartOptions {
	source: AudioSource;
	/** Caption language (translation target). */
	targetLanguage: TargetLanguage;
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

/** Names of events emitted from Rust to the front-end (see commands.rs). */
export const EVT = {
	caption: 'caption',
	level: 'audio-level',
	status: 'status',
} as const;
