import { describe, expect, it } from 'vitest';
import {
	PROVIDER_META,
	estimateSessionCost,
	formatUsd,
	isLocalWhisper,
	modelIdParts,
	modelLabel,
	providerCanTranslate,
	providerDetectsLanguage,
	providerKeyName,
	providerRequiresKey,
	providerTakesGlossary,
	rateText
} from './providers';
import { en } from './i18n/en';
import { PROVIDERS } from './types';

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

// DeepL Voice translates and transcribes in one stream, under its own DeepL API key. It
// serves the Translate mode only: the operator picks the caption language, DeepL hears which
// language is spoken, and it takes no names-and-terms list.
describe('DeepL Voice', () => {
	it('serves the Translate mode only, with its own credential', () => {
		expect(providerCanTranslate('deepl')).toBe(true);
		expect(providerRequiresKey('deepl')).toBe(true);
		expect(providerKeyName('deepl')).toBe('DeepL');
		expect(PROVIDER_META.deepl.keyUrl).toBe('https://www.deepl.com/your-account/keys');
	});

	it('asks for the caption language and takes no glossary', () => {
		expect(providerDetectsLanguage('deepl')).toBe(false);
		expect(providerTakesGlossary('deepl')).toBe(false);
	});

	it('bills each open stream at the reported hourly rate', () => {
		expect(rateText(PROVIDER_META.deepl, en)).toBe('≈ $1.85/hr');
		expect(estimateSessionCost('deepl', 3_600_000, 2)).toBeCloseTo(3.7);
		// No model id to show, so the endpoint stands in for it, whole.
		expect(modelLabel(PROVIDER_META.deepl, en)).toBe('v3/voice/realtime');
		expect(modelIdParts('v3/voice/realtime')).toEqual(['v3/voice/realtime']);
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
		expect(providerDetectsLanguage('deepl')).toBe(false);
		expect(providerDetectsLanguage('ondevice')).toBe(false);
		// Whisper detects what is spoken, but translating, the room reads English, and that is
		// the operator's choice like any other caption language.
		expect(providerDetectsLanguage('whisper-translate')).toBe(false);
	});
});

// The names-and-terms list goes to every engine with a field or a prompt for it.
describe('providerTakesGlossary', () => {
	it('is true for the speech engines and false for the translation models and the demo', () => {
		for (const provider of [
			'mistral',
			'gemini-transcribe',
			'whisper',
			'whisper-translate'
		] as const)
			expect(providerTakesGlossary(provider), provider).toBe(true);
		for (const provider of ['gemini', 'openai', 'deepl', 'ondevice'] as const)
			expect(providerTakesGlossary(provider), provider).toBe(false);
	});
});

// Whisper serves both modes the way Gemini does, as two ids — but one local engine. What the
// id decides is the mode; everything about running the engine follows `isLocalWhisper`.
describe('the two Whisper tasks', () => {
	it('route each to exactly one mode', () => {
		expect(providerCanTranslate('whisper')).toBe(false);
		expect(providerCanTranslate('whisper-translate')).toBe(true);
	});

	it('run the same keyless local engine, and only they do', () => {
		expect(PROVIDERS.filter(isLocalWhisper)).toEqual(['whisper', 'whisper-translate']);
		for (const provider of ['whisper', 'whisper-translate'] as const) {
			expect(providerRequiresKey(provider)).toBe(false);
			expect(rateText(PROVIDER_META[provider], en)).toBe(en.cost.free);
			// Free whatever runs: two sources and an hour cost nothing.
			expect(estimateSessionCost(provider, 3_600_000, 2)).toBe(0);
		}
	});

	it('describes the translating one by what it writes', () => {
		expect(modelLabel(PROVIDER_META['whisper-translate'], en)).toBe(
			en.provider.model['whisper-translate']
		);
		expect(modelLabel(PROVIDER_META.whisper, en)).toBe(en.provider.model.whisper);
	});
});

// Review U11: the running estimate printed `$0.32` in French and German, beside figures those
// languages write with a decimal comma.
describe('the running estimate', () => {
	it('writes dollars the way the interface language writes money', () => {
		expect(formatUsd(3.456, 'en-GB')).toBe('$3.46');
		expect(formatUsd(0, 'en-GB')).toBe('$0.00');
		// The symbol follows the number, after a space that cannot break off on its own line.
		expect(formatUsd(0.32, 'fr-FR')).toMatch(/^0,32[  ]\$$/);
		expect(formatUsd(0.32, 'de-DE')).toMatch(/^0,32[  ]\$$/);
		expect(formatUsd(1234.5, 'de-DE')).toMatch(/^1\.234,50[  ]\$$/);
	});
});

// Review N1: Voxtral's id wrapped with `overflow-wrap: anywhere` and left a lone "0" behind.
describe('where a model id breaks', () => {
	it('is after a hyphen that ends a word, never inside a version number', () => {
		expect(modelIdParts('voxtral-mini-transcribe-realtime-3-0')).toEqual([
			'voxtral-',
			'mini-',
			'transcribe-',
			'realtime-3-0'
		]);
		expect(modelIdParts('gemini-3.5-transcribe-live')).toEqual([
			'gemini-3.5-',
			'transcribe-',
			'live'
		]);
		expect(modelIdParts('gpt-realtime-translate').join('')).toBe('gpt-realtime-translate');
	});

	it('leaves words to wrap at their spaces', () => {
		expect(modelIdParts('Offline · into English only')).toEqual(['Offline · into English only']);
		expect(modelIdParts('')).toEqual(['']);
	});
});
