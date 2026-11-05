// Asking once for each cloud provider before audio is first sent to it (App Review Guideline
// 5.1.2(i); docs/decisions/2026-11-05-mac-app-store.md). The answer is kept per provider,
// not per engine: both Gemini engines read one key and send to one company, so agreeing to
// Google Gemini once covers translation and subtitles. Local Whisper and the built-in demo send
// nothing, so they are never asked about.

import { persistedWith, readStored, writeStored } from './persisted';
import { providerKeyName, providerRequiresKey } from './providers';
import type { Provider } from './types';

/** localStorage key: the providers agreed to, as their key names (`Gemini`, `OpenAI`,
 *  `Mistral`). */
export const AI_CONSENT_KEY = 'privacy.aiConsent';

/** Shown in the question, as text: the operator can read or copy it without leaving the
 *  prompt. */
export const PRIVACY_POLICY_URL =
	'https://github.com/fmadore/Live-translation/blob/main/docs/privacy.md';

const PROVIDERS = new Set(['Gemini', 'OpenAI', 'Mistral']);

/** What was agreed to, validated: storage outlives updates and can hold anything. */
export function loadAiConsent(): string[] {
	try {
		const stored: unknown = JSON.parse(readStored(AI_CONSENT_KEY) ?? '[]');
		if (!Array.isArray(stored)) return [];
		return [
			...new Set(stored.filter((p): p is string => typeof p === 'string' && PROVIDERS.has(p)))
		];
	} catch {
		return [];
	}
}

/** The providers agreed to. Only ever added to; removing a provider's key is how to stop
 *  sending it anything. */
export const aiConsent = persistedWith<string[]>(loadAiConsent, (agreed) =>
	writeStored(AI_CONSENT_KEY, JSON.stringify(agreed))
);

/** Whether starting `provider` needs the question first: a cloud engine whose provider has not
 *  been agreed to. */
export function needsAiConsent(provider: Provider, agreed: readonly string[]): boolean {
	return providerRequiresKey(provider) && !agreed.includes(providerKeyName(provider));
}

/** Record the operator's yes for `provider`'s company. */
export function grantAiConsent(provider: Provider): void {
	if (!providerRequiresKey(provider)) return;
	const name = providerKeyName(provider);
	aiConsent.update((agreed) => (agreed.includes(name) ? agreed : [...agreed, name]));
}
