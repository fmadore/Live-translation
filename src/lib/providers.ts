// Display metadata and the cost model for each caption backend. Rates mirror the table in
// README.md ("Running costs") — update both together.

import type { Provider } from './types';

export interface ProviderMeta {
	id: Provider;
	/** Vendor name shown to the operator. */
	vendor: string;
	/** Exact model id, rendered in the mono face. */
	modelId: string;
	/** Rate as published, including the range where billing is not flat. */
	hourlyText: string;
	/** Single figure used for the running meter; the midpoint where the rate is a range. */
	hourlyEstimate: number;
	/** True when the rate is billed per open stream, so "Both" sources doubles it. */
	perStream: boolean;
	/** Where to get an API key; null for backends that need none. */
	keyUrl: string | null;
	/** One-sentence explanation of what the backend actually does. */
	blurb: string;
}

export const PROVIDER_META: Record<Provider, ProviderMeta> = {
	gemini: {
		id: 'gemini',
		vendor: 'Google Gemini',
		modelId: 'gemini-3.5-live-translate-preview',
		hourlyText: '$1.25–2.21/hr',
		hourlyEstimate: 1.73,
		perStream: true,
		keyUrl: 'https://aistudio.google.com/apikey',
		blurb:
			'Translated captions come from the output transcription; the generated audio is discarded.'
	},
	openai: {
		id: 'openai',
		vendor: 'OpenAI',
		modelId: 'gpt-realtime-translate',
		hourlyText: '$3.06/hr',
		hourlyEstimate: 3.06,
		perStream: true,
		keyUrl: 'https://platform.openai.com/api-keys',
		blurb:
			"OpenAI's dedicated live speech-translation model (70+ languages in, 13 out). Captions come from its transcript; audio is discarded."
	},
	mistral: {
		id: 'mistral',
		vendor: 'Mistral Voxtral',
		modelId: 'voxtral-mini-transcribe-realtime-2602',
		hourlyText: '$0.36/hr',
		hourlyEstimate: 0.36,
		perStream: true,
		keyUrl: 'https://console.mistral.ai/api-keys',
		blurb: '16 kHz realtime speech-to-text with a 480 ms target delay.'
	},
	ondevice: {
		id: 'ondevice',
		vendor: 'Built-in demo',
		modelId: 'Bundled sample · deterministic',
		hourlyText: 'Free',
		hourlyEstimate: 0,
		perStream: false,
		keyUrl: null,
		blurb: 'Demonstrates captions, overlay, timing and export without capturing or recognizing live audio.'
	}
};

/** Cost accrued so far, in USD. Per-stream providers bill once per open source, so "Both"
 *  doubles the rate; the built-in demonstration bills nothing. */
export function estimateSessionCost(
	provider: Provider,
	elapsedMs: number,
	streams: 1 | 2
): number {
	const meta = PROVIDER_META[provider];
	const hours = elapsedMs / 3_600_000;
	return meta.hourlyEstimate * hours * (meta.perStream ? streams : 1);
}

export function formatUsd(n: number): string {
	return '$' + n.toFixed(2);
}
