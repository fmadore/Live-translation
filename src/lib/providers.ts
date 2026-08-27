// Display metadata and the cost model for each caption backend. Rates mirror the table in
// README.md ("Running costs") — update both together.

import type { Messages } from './i18n/en';
import type { Provider } from './types';

export interface ProviderMeta {
	id: Provider;
	/** Exact model id, rendered in the mono face. Not translated — it is an identifier.
	 *  The built-in demonstration has no model, so it describes itself from the catalog. */
	modelId: string;
	/** Rate as published, including the range where billing is not flat. Null where there is
	 *  nothing to bill; the word for that belongs to the interface language, not here. */
	hourlyRate: string | null;
	/** Single figure used for the running meter; the midpoint where the rate is a range. */
	hourlyEstimate: number;
	/** True when the rate is billed per open stream, so "Both" sources doubles it. */
	perStream: boolean;
	/** Where to get an API key; null for backends that need none. */
	keyUrl: string | null;
}

export const PROVIDER_META: Record<Provider, ProviderMeta> = {
	gemini: {
		id: 'gemini',
		modelId: 'gemini-3.5-live-translate-preview',
		hourlyRate: '$1.25–2.21',
		hourlyEstimate: 1.73,
		perStream: true,
		keyUrl: 'https://aistudio.google.com/apikey'
	},
	'gemini-transcribe': {
		id: 'gemini-transcribe',
		modelId: 'gemini-3.5-transcribe-live',
		hourlyRate: '$0.30–0.54',
		hourlyEstimate: 0.42,
		perStream: true,
		keyUrl: 'https://aistudio.google.com/apikey'
	},
	openai: {
		id: 'openai',
		modelId: 'gpt-realtime-translate',
		hourlyRate: '$3.06',
		hourlyEstimate: 3.06,
		perStream: true,
		keyUrl: 'https://platform.openai.com/api-keys'
	},
	mistral: {
		id: 'mistral',
		modelId: 'voxtral-mini-transcribe-realtime-2602',
		hourlyRate: '$0.36',
		hourlyEstimate: 0.36,
		perStream: true,
		keyUrl: 'https://console.mistral.ai/api-keys'
	},
	ondevice: {
		id: 'ondevice',
		modelId: '',
		hourlyRate: null,
		hourlyEstimate: 0,
		perStream: false,
		keyUrl: null
	}
};

/** Cost accrued so far, in USD. Per-stream providers bill once per open source, so "Both"
 *  doubles the rate; the built-in demonstration bills nothing. */
export function estimateSessionCost(provider: Provider, elapsedMs: number, streams: 1 | 2): number {
	const meta = PROVIDER_META[provider];
	const hours = elapsedMs / 3_600_000;
	return meta.hourlyEstimate * hours * (meta.perStream ? streams : 1);
}

export function formatUsd(n: number): string {
	return '$' + n.toFixed(2);
}

/** The published rate, split so the unit can be dimmed: `['$3.06', '/hr']`, or the word for
 *  free and nothing to dim. */
export function rateParts(meta: ProviderMeta, m: Messages): [string, string] {
	return meta.hourlyRate === null ? [m.cost.free, ''] : [meta.hourlyRate, m.cost.perHour];
}

/** The rate as one string, for the pre-flight row. */
export function rateText(meta: ProviderMeta, m: Messages): string {
	return rateParts(meta, m).join('');
}

/** What to print under the vendor name. Real backends have a model id; the demonstration has
 *  a description instead, and that is prose. */
export function modelLabel(meta: ProviderMeta, m: Messages): string {
	return meta.modelId || m.provider.model.ondevice;
}
