// Svelte stores shared across the operator window. The overlay window keeps its own
// minimal state (see routes/overlay/+page.svelte) so it stays lightweight.

import { writable, derived } from 'svelte/store';
import type { AudioLevel, Caption, SessionState, StartOptions, TranscriptLine } from './types';
import { DEFAULT_OVERLAY_FONT, OVERLAY_FONT_KEY } from './types';

export const sessionState = writable<SessionState>('idle');
export const statusMessage = writable<string>('');

export const hasKey = writable<boolean>(false);

export const options = writable<StartOptions>({
	source: 'system',
	targetLanguage: 'en',
	mode: 'live-translate',
	micDeviceName: null
});

// Latest caption per origin, for the operator monitor.
export const latestCaption = writable<Caption | null>(null);

// Rolling transcript log (most recent finalized lines first).
export const transcript = writable<TranscriptLine[]>([]);

export function pushCaption(c: Caption) {
	latestCaption.set(c);
	if (c.final && c.text.trim()) {
		const line: TranscriptLine = {
			time: new Date().toLocaleTimeString(),
			text: c.text.trim(),
			sourceText: c.sourceText.trim(),
			origin: c.origin
		};
		transcript.update((list) => [line, ...list].slice(0, 1000));
	}
}

// Overlay font size, persisted to localStorage so both windows share the same default.
function loadOverlayFont(): number {
	if (typeof localStorage === 'undefined') return DEFAULT_OVERLAY_FONT;
	const v = Number(localStorage.getItem(OVERLAY_FONT_KEY));
	return Number.isFinite(v) && v > 0 ? v : DEFAULT_OVERLAY_FONT;
}

export const overlayFontSize = writable<number>(loadOverlayFont());

overlayFontSize.subscribe((v) => {
	if (typeof localStorage !== 'undefined') localStorage.setItem(OVERLAY_FONT_KEY, String(v));
});

// Audio levels keyed by source.
export const micLevel = writable<AudioLevel>({ source: 'microphone', rms: 0, peak: 0 });
export const systemLevel = writable<AudioLevel>({ source: 'system', rms: 0, peak: 0 });

export const isRunning = derived(sessionState, ($s) => $s === 'running' || $s === 'reconnecting');
