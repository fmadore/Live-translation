import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { get } from 'svelte/store';
import {
	AI_CONSENT_KEY,
	aiConsent,
	grantAiConsent,
	loadAiConsent,
	needsAiConsent
} from './aiConsent';
import { providerKeyName, providerRequiresKey } from './providers';
import { PROVIDERS } from './types';

// The logic project runs in node, which has no localStorage: a memory one stands in.
const stored = new Map<string, string>();

beforeEach(() => {
	stored.clear();
	Object.defineProperty(globalThis, 'localStorage', {
		configurable: true,
		value: {
			getItem: (key: string) => stored.get(key) ?? null,
			setItem: (key: string, value: string) => void stored.set(key, value)
		}
	});
	aiConsent.set([]);
});

afterEach(() => {
	Reflect.deleteProperty(globalThis, 'localStorage');
});

describe('the provider question', () => {
	it('is never asked for an engine that sends nothing', () => {
		for (const provider of ['whisper', 'whisper-translate', 'ondevice'] as const)
			expect(needsAiConsent(provider, [])).toBe(false);
	});

	it('is asked once per company, so both Gemini engines share one answer', () => {
		expect(needsAiConsent('gemini', [])).toBe(true);
		grantAiConsent('gemini-transcribe');
		expect(get(aiConsent)).toEqual(['Gemini']);
		expect(needsAiConsent('gemini', get(aiConsent))).toBe(false);
		expect(needsAiConsent('openai', get(aiConsent))).toBe(true);
		grantAiConsent('gemini');
		expect(get(aiConsent)).toEqual(['Gemini']);
	});

	it('asks about DeepL on its own, and reads its answer back', () => {
		grantAiConsent('openai');
		expect(needsAiConsent('deepl', get(aiConsent))).toBe(true);
		grantAiConsent('deepl');
		expect(needsAiConsent('deepl', get(aiConsent))).toBe(false);
		expect(loadAiConsent()).toEqual(['OpenAI', 'DeepL']);
	});

	// Every cloud engine's company has to survive a reload, or the question comes back each
	// launch.
	it('knows every company a cloud engine sends audio to', () => {
		for (const provider of PROVIDERS.filter(providerRequiresKey)) {
			stored.set(AI_CONSENT_KEY, JSON.stringify([providerKeyName(provider)]));
			expect(loadAiConsent(), provider).toEqual([providerKeyName(provider)]);
		}
	});

	it('keeps the answer, and reads back only the providers it knows', () => {
		grantAiConsent('mistral');
		expect(JSON.parse(stored.get(AI_CONSENT_KEY) ?? '')).toEqual(['Mistral']);
		stored.set(AI_CONSENT_KEY, JSON.stringify(['OpenAI', 'Elsewhere', 3, 'OpenAI']));
		expect(loadAiConsent()).toEqual(['OpenAI']);
		stored.set(AI_CONSENT_KEY, '{not json');
		expect(loadAiConsent()).toEqual([]);
	});
});
