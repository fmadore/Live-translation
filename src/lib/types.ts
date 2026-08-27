// Shared types between the operator window, the caption overlay, and the Rust core.
// These mirror the serde structs in `src-tauri/src/types.rs` — keep them in sync.

/** Which audio input(s) to translate or transcribe. */
export type AudioSource = 'microphone' | 'system' | 'both';

/** A single capture source — the `origin` on captions, levels, and status updates. */
export type Origin = 'microphone' | 'system';

/** BCP-47 codes we use for the two caption languages. The spoken language is auto-detected. */
export type TargetLanguage = 'en' | 'fr';

/** Caption backend. The commercial providers each have their own API key; `ondevice`
 *  is the bundled product demonstration and needs no credential. Mirrors `Provider` in types.rs.
 *
 *  `gemini` and `gemini-transcribe` are two different Google models sharing one endpoint and
 *  one key — Live Translate and Transcribe Live. They are separate ids because they serve
 *  different modes at different rates, and `providerCanTranslate` has to stay a plain
 *  function of the provider. */
export type Provider = 'gemini' | 'gemini-transcribe' | 'openai' | 'mistral' | 'ondevice';

export interface OnDeviceReadiness {
	ready: boolean;
	engine: 'built-in-demo' | 'none' | string;
	state: string;
	canPrepare: boolean;
	detail: string;
}

/** Backends that produce translated captions. The built-in demo is same-language only. */
export function providerCanTranslate(provider: Provider): boolean {
	return provider === 'gemini' || provider === 'openai';
}

/** Whether an API key must be saved before a session can start. The built-in demo is the
 *  one backend that works with no credential, which keeps provider keys out of the
 *  app's primary functionality — see `docs/microsoft-store.md`. */
export function providerRequiresKey(provider: Provider): boolean {
	return provider !== 'ondevice';
}

/** Which credential a backend reads. Both Gemini models share one AI Studio key, so saving
 *  it once covers translation and subtitles. Mirrors `account()` in `src-tauri/src/secrets.rs`. */
export function providerKeyName(provider: Provider): string {
	if (provider === 'openai') return 'OpenAI';
	if (provider === 'mistral') return 'Mistral';
	return 'Gemini';
}

/** Whether the backend identifies the spoken language itself, so there is no language for
 *  the operator to choose. True for both subtitle engines; the built-in demo instead picks
 *  which bundled script to play. */
export function providerDetectsLanguage(provider: Provider): boolean {
	return provider === 'mistral' || provider === 'gemini-transcribe';
}

/** Translate speech, or show a same-language transcription as live subtitles. */
export type OutputMode = 'translate' | 'transcribe';

/** Whether the F2 direction shortcut can act right now.
 *
 *  Translation is the only mode with a direction to flip, and the target language is handed
 *  to the provider once at session start — so it cannot change while a session is running,
 *  and `locked` is true for the whole of one. Genuine live switching needs provider-aware
 *  reconnection, hysteresis and in-flight-turn handling; that is issue #12, not this. */
export function canFlipDirection(mode: OutputMode, locked: boolean): boolean {
	return mode === 'translate' && !locked;
}

export interface StartOptions {
	source: AudioSource;
	mode: OutputMode;
	/** Caption language (translation target). */
	targetLanguage: TargetLanguage;
	/** Realtime translation/transcription backend. */
	provider: Provider;
	/** Input device name for the microphone; null = system default. */
	micDeviceName?: string | null;
	/** Rehearse instead of capturing: a cloud backend plays a bundled ~20 s speech fixture spoken in
	 *  this language through the real pipeline — one System-origin stream, looping until Stop —
	 *  so captions, levels, transcript and overlay behave exactly as in a live session. `source`
	 *  and `micDeviceName` are ignored while it is set; absent means a normal live session.
	 *  Cloud engines still stream to the provider and bill normally. The built-in demo already
	 *  has its own deterministic timeline and disables this separate rehearsal control. Per-launch only: it is never written to the options store, so it can never reach
	 *  the persisted record. Keep in sync with `StartOptions` in `src-tauri/src/types.rs`. */
	rehearsal?: TargetLanguage;
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

/** Preflight level test. Deliberately not a `StatusUpdate`: a test is not a session and must
 *  never move the session state machine. `message` is set only when a device failed.
 *  Mirrors `AudioTestUpdate` in types.rs. */
export interface AudioTestUpdate {
	active: boolean;
	message?: string | null;
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

/** A recovery file as the Rust core hands it over: the raw JSON it read, plus where it sits
 *  so the operator can be told exactly which file to delete. Mirrors `StoredRecovery` in
 *  `src-tauri/src/recovery.rs`. Parsing is the front-end's job (`decodeRecovery`) — the core
 *  never interprets caption text. */
export interface StoredRecovery {
	path: string;
	contents: string;
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

/** A tray menu entry the front-end has to carry out, because it needs session or transcript
 *  state that only the renderer holds. *Open* is absent on purpose: showing a window needs
 *  nothing from here, so the core does it itself and the menu keeps working even if this
 *  window is wedged. Mirrors `TrayCommand` in `src-tauri/src/tray.rs`. */
export type TrayCommand = 'toggle-overlay' | 'stop-session' | 'quit';

/** Event names. Rust→front-end: caption/level/status/closeRequested/trayCommand.
 *  Operator→overlay: overlayConfig. Overlay→operator: overlayState. */
export const EVT = {
	caption: 'caption',
	level: 'audio-level',
	status: 'status',
	audioTest: 'audio-test',
	closeRequested: 'close-requested',
	trayCommand: 'tray-command',
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

/** localStorage key for the opt-in crash-recovery spool. Absent means off, which is the
 *  privacy-first default: nothing is written to disk unless the operator asks for it. */
export const RECOVERY_ENABLED_KEY = 'recovery.enabled';

export function loadRecoveryEnabled(): boolean {
	if (typeof localStorage === 'undefined') return false;
	return localStorage.getItem(RECOVERY_ENABLED_KEY) === 'true';
}

/** Whether closing the operator window leaves the app running in the tray.
 *
 *  Off by default, so a fresh install keeps ordinary Windows semantics: minimize goes to the
 *  taskbar, and the X closes the app. An app that silently keeps running after you closed it
 *  is a thing you opt into. */
export const CLOSE_TO_TRAY_KEY = 'window.closeToTray';

export function loadCloseToTray(): boolean {
	if (typeof localStorage === 'undefined') return false;
	return localStorage.getItem(CLOSE_TO_TRAY_KEY) === 'true';
}

/** Set once the operator has been told, in as many words, that closing the window is no
 *  longer quitting. Persisted so it is said the first time and never again. */
export const TRAY_HIDE_EXPLAINED_KEY = 'window.trayHideExplained';

export function loadTrayHideExplained(): boolean {
	if (typeof localStorage === 'undefined') return false;
	return localStorage.getItem(TRAY_HIDE_EXPLAINED_KEY) === 'true';
}

/** Fresh-install session setup. The bundled demonstration needs no hardware, network, account,
 *  or API key and is transparently identified as a demonstration. */
export const DEFAULT_START_OPTIONS: StartOptions = {
	source: 'microphone',
	mode: 'transcribe',
	targetLanguage: 'en',
	provider: 'ondevice',
	micDeviceName: null
};

/** localStorage key for the operator's last setup, so the keyless default above is a first-run
 *  state rather than a reset on every launch. */
export const SESSION_OPTIONS_KEY = 'session.options';

// Runtime members of the unions declared above; persisted values are untrusted input, so each
// field is checked against its list. Keep these in step with the types.
const AUDIO_SOURCES: readonly AudioSource[] = ['microphone', 'system', 'both'];
const OUTPUT_MODES: readonly OutputMode[] = ['translate', 'transcribe'];
const TARGET_LANGUAGES: readonly TargetLanguage[] = ['en', 'fr'];
const PROVIDERS: readonly Provider[] = [
	'gemini',
	'gemini-transcribe',
	'openai',
	'mistral',
	'ondevice'
];

function oneOf<T extends string>(allowed: readonly T[], value: unknown, fallback: T): T {
	return typeof value === 'string' && (allowed as readonly string[]).includes(value)
		? (value as T)
		: fallback;
}

/** Read the persisted setup. Absent, unparseable or non-object storage yields the first-run
 *  defaults; otherwise each field falls back to its own default when missing or outside its
 *  union. A stored mode/provider pair that violates `providerCanTranslate` discards the whole
 *  record — the rail offers no such pair, so repairing one field would only guess which of the
 *  two the operator meant. The result is built field by field rather than spread from storage,
 *  so `rehearsal` (never persisted, and meaningless outside the launch that asked for it) can
 *  never come back out of localStorage. */
export function loadStartOptions(): StartOptions {
	if (typeof localStorage === 'undefined') return { ...DEFAULT_START_OPTIONS };
	const raw = localStorage.getItem(SESSION_OPTIONS_KEY);
	if (!raw) return { ...DEFAULT_START_OPTIONS };
	let parsed: unknown;
	try {
		parsed = JSON.parse(raw);
	} catch {
		return { ...DEFAULT_START_OPTIONS };
	}
	if (typeof parsed !== 'object' || parsed === null) return { ...DEFAULT_START_OPTIONS };
	const stored = parsed as Record<string, unknown>;
	const loaded: StartOptions = {
		source: oneOf(AUDIO_SOURCES, stored.source, DEFAULT_START_OPTIONS.source),
		mode: oneOf(OUTPUT_MODES, stored.mode, DEFAULT_START_OPTIONS.mode),
		targetLanguage: oneOf(
			TARGET_LANGUAGES,
			stored.targetLanguage,
			DEFAULT_START_OPTIONS.targetLanguage
		),
		provider: oneOf(PROVIDERS, stored.provider, DEFAULT_START_OPTIONS.provider),
		micDeviceName: typeof stored.micDeviceName === 'string' ? stored.micDeviceName : null
	};
	// The compatibility id `ondevice` now means the deterministic bundled demonstration.
	// Repair older saved Windows-speech configurations to its single virtual Demo audio source.
	if (loaded.provider === 'ondevice') {
		loaded.source = 'microphone';
		loaded.micDeviceName = null;
	}
	return providerCanTranslate(loaded.provider) === (loaded.mode === 'translate')
		? loaded
		: { ...DEFAULT_START_OPTIONS };
}
