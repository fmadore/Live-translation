import { get } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createSetupActions } from './setupActions';
import { languageFavourites, options } from './stores';
import type { StartOptions } from './types';

const base: StartOptions = {
	source: 'both',
	mode: 'translate',
	provider: 'gemini',
	targetLanguage: 'fr',
	micDeviceName: 'Room mic',
	micDeviceId: null
};

function setup(locked = false) {
	const deps = {
		locked: vi.fn(() => locked),
		invalidateAudioTest: vi.fn(),
		refreshDevices: vi.fn()
	};
	return { actions: createSetupActions(deps), deps };
}

beforeEach(() => {
	options.set({ ...base });
	languageFavourites.set(['en', 'fr']);
});

describe('setup actions', () => {
	it('refuses every change while setup is locked', () => {
		const { actions, deps } = setup(true);
		actions.setMode('transcribe');
		actions.setSource('microphone');
		actions.setProvider('openai');
		actions.setTarget('de');
		actions.flipDirection();
		expect(get(options)).toEqual(base);
		expect(deps.invalidateAudioTest).not.toHaveBeenCalled();
	});

	it('switching mode picks that mode default engine and resets a transcription to the mic', () => {
		const { actions, deps } = setup();
		actions.setMode('transcribe');
		expect(get(options)).toMatchObject({
			mode: 'transcribe',
			provider: 'ondevice',
			source: 'microphone',
			micDeviceName: null
		});
		actions.setMode('translate');
		expect(get(options)).toMatchObject({ mode: 'translate', provider: 'gemini' });
		expect(deps.invalidateAudioTest).toHaveBeenCalledTimes(2);
	});

	it('only accepts engines that serve the current mode, and re-lists devices for them', () => {
		const { actions, deps } = setup();
		actions.setProvider('mistral');
		expect(get(options).provider).toBe('gemini');
		actions.setProvider('openai');
		expect(get(options).provider).toBe('openai');
		expect(deps.refreshDevices).toHaveBeenCalledOnce();
	});

	it('forgets an audio check only when the source actually changes', () => {
		const { actions, deps } = setup();
		actions.setSource('both');
		expect(deps.invalidateAudioTest).not.toHaveBeenCalled();
		actions.setSource('system');
		expect(get(options).source).toBe('system');
		expect(deps.invalidateAudioTest).toHaveBeenCalledOnce();
	});

	it('keeps the built-in demo on its bundled sample', () => {
		options.set({ ...base, mode: 'transcribe', provider: 'ondevice', source: 'microphone' });
		const { actions } = setup();
		actions.setSource('system');
		expect(get(options).source).toBe('microphone');
		actions.setTarget('fr');
		expect(get(options).targetLanguage).toBe('fr');
	});

	it('leaves the language alone for engines that detect it', () => {
		options.set({ ...base, mode: 'transcribe', provider: 'mistral', targetLanguage: 'en' });
		const { actions } = setup();
		actions.setTarget('fr');
		expect(get(options).targetLanguage).toBe('en');
	});

	it('flips between the first two favourites the engine supports', () => {
		const { actions } = setup();
		actions.flipDirection();
		expect(get(options).targetLanguage).toBe('en');
		actions.flipDirection();
		expect(get(options).targetLanguage).toBe('fr');
	});
});
