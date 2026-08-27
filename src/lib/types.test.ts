import { afterEach, describe, expect, it } from 'vitest';
import {
	canFlipDirection,
	loadStartOptions,
	providerCanTranslate,
	providerDetectsLanguage,
	providerKeyName,
	providerRequiresKey,
	SESSION_OPTIONS_KEY
} from './types';
import { PROVIDER_META } from './providers';

// The operator UI used to offer "Flip mid-session with F2" over a handler that refused for
// the whole of a session (issue #21). Keeping the rule in one tested predicate is what stops
// the shortcut and the copy drifting apart again.
describe('canFlipDirection', () => {
	it('allows a swap while the operator is still setting a translation up', () => {
		expect(canFlipDirection('translate', false)).toBe(true);
	});

	it('refuses once controls are locked, which is the whole of a running session', () => {
		expect(canFlipDirection('translate', true)).toBe(false);
	});

	it('refuses in subtitle mode, which has no direction to flip', () => {
		expect(canFlipDirection('transcribe', false)).toBe(false);
		expect(canFlipDirection('transcribe', true)).toBe(false);
	});
});

// Gemini appears twice under two different model ids — Live Translate for captions,
// Transcribe Live for subtitles. These predicates are what keep the two apart everywhere:
// the rail that offers a backend, the guard that starts a session, and the credential row.
describe('the two Gemini backends', () => {
	it('routes each to exactly one mode', () => {
		expect(providerCanTranslate('gemini')).toBe(true);
		expect(providerCanTranslate('gemini-transcribe')).toBe(false);
	});

	it('shares one AI Studio credential, so switching mode never re-prompts', () => {
		expect(providerKeyName('gemini-transcribe')).toBe(providerKeyName('gemini'));
		expect(PROVIDER_META['gemini-transcribe'].keyUrl).toBe(PROVIDER_META.gemini.keyUrl);
		expect(providerRequiresKey('gemini-transcribe')).toBe(true);
	});

	it('bills the subtitle model separately from the translation model', () => {
		expect(PROVIDER_META['gemini-transcribe'].modelId).toBe('gemini-3.5-transcribe-live');
		expect(PROVIDER_META['gemini-transcribe'].hourlyEstimate).toBeLessThan(
			PROVIDER_META.gemini.hourlyEstimate
		);
	});
});

// Step 03 asks for a language only when the operator has one to give. Both subtitle engines
// identify the spoken language themselves; the built-in demo instead picks which script to play.
describe('providerDetectsLanguage', () => {
	it('is true for the backends that identify the spoken language themselves', () => {
		expect(providerDetectsLanguage('mistral')).toBe(true);
		expect(providerDetectsLanguage('gemini-transcribe')).toBe(true);
	});

	it('is false where the operator still chooses one', () => {
		expect(providerDetectsLanguage('gemini')).toBe(false);
		expect(providerDetectsLanguage('openai')).toBe(false);
		expect(providerDetectsLanguage('ondevice')).toBe(false);
	});
});

// This project runs in node, so the persisted-setup path needs a stand-in for the one
// storage method `loadStartOptions` reads.
function storing(record: unknown): void {
	Object.defineProperty(globalThis, 'localStorage', {
		configurable: true,
		value: { getItem: () => JSON.stringify(record) }
	});
}

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
		expect(loadStartOptions().provider).toBe('ondevice');
	});
});
