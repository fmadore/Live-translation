// The setup sheet's choices: mode, audio source, caption language and engine, plus the F2
// language flip. Each one is refused while setup is locked, and each keeps the options a
// combination the core accepts, so the sheet never has to know those rules itself.

import { get } from 'svelte/store';
import { nextFavourite } from './languages';
import { languageFavourites, options } from './stores';
import { canFlipDirection, providerCanTranslate } from './types';
import type { AudioSource, OutputMode, Provider, TargetLanguage } from './types';

export interface SetupDeps {
	/** A session, a start or a profile load is under way. */
	locked: () => boolean;
	/** Forget a passed audio check, because what it tested has changed. */
	invalidateAudioTest: () => void;
	/** Re-list devices after switching to an engine that captures them. */
	refreshDevices: () => void;
}

export function createSetupActions({ locked, invalidateAudioTest, refreshDevices }: SetupDeps) {
	function setSource(source: AudioSource) {
		const current = get(options);
		if (locked()) return;
		if (current.provider === 'ondevice' && source !== 'microphone') return;
		if (source !== current.source) invalidateAudioTest();
		options.set({ ...current, source });
	}

	function setTarget(targetLanguage: TargetLanguage) {
		const current = get(options);
		if (locked()) return;
		// Translation picks the language the room reads; the built-in demo picks its script.
		// The subtitle engines auto-detect, so they have nothing to set.
		if (current.mode !== 'translate' && current.provider !== 'ondevice') return;
		options.set({ ...current, targetLanguage });
	}

	function setProvider(provider: Provider) {
		const current = get(options);
		if (locked() || provider === current.provider) return;
		// Each mode accepts only the backends that can serve it.
		if (providerCanTranslate(provider) !== (current.mode === 'translate')) return;
		invalidateAudioTest();
		options.set({
			...current,
			provider,
			...(provider === 'ondevice' ? { source: 'microphone' as const, micDeviceName: null } : {})
		});
		if (provider !== 'ondevice') refreshDevices();
	}

	function setMode(mode: OutputMode) {
		const current = get(options);
		if (locked() || mode === current.mode) return;
		invalidateAudioTest();
		options.set({
			...current,
			mode,
			provider: mode === 'transcribe' ? 'ondevice' : 'gemini',
			...(mode === 'transcribe' ? { source: 'microphone' as const, micDeviceName: null } : {})
		});
	}

	/** Quick flip of the caption language — handy when speakers alternate. */
	function flipDirection() {
		const current = get(options);
		if (!canFlipDirection(current.mode, locked())) return;
		const next = nextFavourite(current.targetLanguage, get(languageFavourites), current.provider);
		if (next) setTarget(next);
	}

	return { setSource, setTarget, setProvider, setMode, flipDirection };
}
export type SetupActions = ReturnType<typeof createSetupActions>;
