// Svelte stores shared across the operator window. The overlay window keeps its own
// minimal state (see routes/overlay/+page.svelte) so it stays lightweight.

import { writable, derived } from 'svelte/store';
import type { AudioLevel, Caption, SessionState, StartOptions } from './types';

export const sessionState = writable<SessionState>('idle');
export const statusMessage = writable<string>('');

export const hasKey = writable<boolean>(false);

export const options = writable<StartOptions>({
	source: 'system',
	targetLanguage: 'en',
	micDeviceName: null
});

// Latest caption per origin, for the operator monitor.
export const latestCaption = writable<Caption | null>(null);

// Rolling transcript log (most recent finalized captions first).
export const transcript = writable<Caption[]>([]);

export function pushCaption(c: Caption) {
	latestCaption.set(c);
	if (c.final && c.text.trim()) {
		transcript.update((list) => [c, ...list].slice(0, 200));
	}
}

// Audio levels keyed by source.
export const micLevel = writable<AudioLevel>({ source: 'microphone', rms: 0, peak: 0 });
export const systemLevel = writable<AudioLevel>({ source: 'system', rms: 0, peak: 0 });

export const isRunning = derived(sessionState, ($s) => $s === 'running' || $s === 'reconnecting');
