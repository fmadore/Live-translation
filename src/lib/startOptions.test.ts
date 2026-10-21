import { afterEach, describe, expect, it } from 'vitest';
import { DEFAULT_START_OPTIONS, loadStartOptions, normalizeStartOptions } from './startOptions';

// This project runs in node, so the persisted-setup path needs a stand-in for the one
// storage method `loadStartOptions` reads.
function storing(record: unknown): void {
	Object.defineProperty(globalThis, 'localStorage', {
		configurable: true,
		value: { getItem: () => JSON.stringify(record) }
	});
}

it('restores application scope without restoring an old process identity', () => {
	storing({
		source: 'both',
		mode: 'translate',
		provider: 'gemini',
		targetLanguage: 'en',
		systemCapture: { kind: 'application', process: { pid: 123, createdAt: '987' } }
	});
	expect(loadStartOptions().systemCapture).toEqual({ kind: 'application', process: null });
	Reflect.deleteProperty(globalThis, 'localStorage');
});

describe('loadStartOptions with the Gemini subtitle backend', () => {
	afterEach(() => {
		Reflect.deleteProperty(globalThis, 'localStorage');
	});

	it('restores it, because it is a valid pairing with subtitle mode', () => {
		storing({
			source: 'system',
			mode: 'transcribe',
			targetLanguage: 'en',
			provider: 'gemini-transcribe',
			micDeviceName: null
		});
		const loaded = loadStartOptions();
		expect(loaded.provider).toBe('gemini-transcribe');
		expect(loaded.mode).toBe('transcribe');
		expect(loaded.source).toBe('system');
	});

	// The rail offers no such pair, so repairing one field would only guess which of the two
	// the operator meant — the whole record goes back to the keyless first-run defaults.
	it('discards a stored record that pairs it with translation', () => {
		storing({
			source: 'microphone',
			mode: 'translate',
			targetLanguage: 'fr',
			provider: 'gemini-transcribe'
		});
		expect(loadStartOptions().provider).toBe('whisper');
	});
});

describe('loadStartOptions with Whisper translating into English', () => {
	afterEach(() => {
		Reflect.deleteProperty(globalThis, 'localStorage');
	});

	it('restores it with Whisper’s own choices, paired with translation', () => {
		storing({
			source: 'microphone',
			mode: 'translate',
			targetLanguage: 'en',
			provider: 'whisper-translate',
			spokenLanguage: 'fr'
		});
		expect(loadStartOptions()).toMatchObject({
			mode: 'translate',
			provider: 'whisper-translate',
			whisperModel: 'base',
			spokenLanguage: 'fr'
		});
	});

	it('discards a stored record that pairs it with subtitles', () => {
		storing({ mode: 'transcribe', targetLanguage: 'en', provider: 'whisper-translate' });
		expect(loadStartOptions()).toMatchObject({ mode: 'transcribe', provider: 'whisper' });
	});
});

describe('first-launch setup', () => {
	afterEach(() => {
		Reflect.deleteProperty(globalThis, 'localStorage');
	});

	it.each([null, '{invalid', 'null'])(
		'opens Whisper when saved setup is absent or unusable: %s',
		(raw) => {
			Object.defineProperty(globalThis, 'localStorage', {
				configurable: true,
				value: { getItem: () => raw }
			});
			expect(loadStartOptions()).toMatchObject({
				mode: 'transcribe',
				provider: 'whisper',
				source: 'microphone',
				whisperModel: 'base',
				spokenLanguage: null
			});
		}
	);

	it('preserves an existing explicit scripted-demo choice', () => {
		storing({
			mode: 'transcribe',
			provider: 'ondevice',
			source: 'microphone',
			targetLanguage: 'fr'
		});
		expect(loadStartOptions()).toMatchObject({
			provider: 'ondevice',
			targetLanguage: 'fr'
		});
	});
});

describe('Voxtral’s names and terms', () => {
	afterEach(() => {
		Reflect.deleteProperty(globalThis, 'localStorage');
	});

	const voxtral = { mode: 'transcribe', provider: 'mistral', source: 'system' } as const;

	it('come back with the saved setup as they were saved', () => {
		const glossary = ['UNESCO', 'Université Joseph Ki-Zerbo', 'Ouagadougou'];
		storing({ ...voxtral, targetLanguage: 'en', glossary });
		const loaded = loadStartOptions();
		expect(loaded).toMatchObject({ provider: 'mistral', glossary });
		// The setup is stored as the options store holds it, and read back unchanged.
		expect(normalizeStartOptions(JSON.parse(JSON.stringify(loaded)))).toEqual(loaded);
	});

	it('come back trimmed and without anything the core would refuse', () => {
		expect(
			normalizeStartOptions({
				...voxtral,
				glossary: [' UNESCO ', 7, '', 'unesco', 'x'.repeat(201), null, 'Ki-Zerbo']
			}).glossary
		).toEqual(['UNESCO', 'Ki-Zerbo']);
	});

	it('are kept with another engine selected, for when Voxtral is chosen again', () => {
		expect(
			normalizeStartOptions({ mode: 'transcribe', provider: 'whisper', glossary: ['UNESCO'] })
		).toMatchObject({ provider: 'whisper', glossary: ['UNESCO'] });
		expect(
			normalizeStartOptions({ mode: 'translate', provider: 'gemini', glossary: ['UNESCO'] })
		).toMatchObject({ provider: 'gemini', glossary: ['UNESCO'] });
	});

	it('stay absent when none were saved, or only unusable ones', () => {
		for (const glossary of [undefined, [], ['', '  '], 'UNESCO', { 0: 'UNESCO' }]) {
			const loaded = normalizeStartOptions({ ...voxtral, glossary });
			expect(loaded).not.toHaveProperty('glossary');
		}
		expect(DEFAULT_START_OPTIONS).not.toHaveProperty('glossary');
	});
});
