import { describe, expect, it } from 'vitest';
import { billedAt, ledgerCost, runTracks } from './billing';
import { DEFAULT_START_OPTIONS } from './startOptions';
import type { StartOptions } from './types';

const HOUR = 3_600_000;
const gemini: StartOptions = {
	...DEFAULT_START_OPTIONS,
	mode: 'translate',
	provider: 'gemini',
	targetLanguage: 'fr'
};

describe('the cost ledger', () => {
	it('bills every stream at the setup’s rate while nothing changes, as the estimate always did', () => {
		expect(ledgerCost(['microphone', 'system'], 'gemini', [], HOUR)).toBeCloseTo(2 * 1.73);
	});

	it('stops billing a stream that ended, and bills it again when it runs again', () => {
		const changes = [
			{ track: 'system' as const, provider: null, atMs: HOUR / 2 },
			{ track: 'system' as const, provider: 'gemini' as const, atMs: HOUR }
		];
		expect(ledgerCost(['system'], 'gemini', changes, HOUR)).toBeCloseTo(1.73 / 2);
		expect(ledgerCost(['system'], 'gemini', changes, 2 * HOUR)).toBeCloseTo(1.73 * 1.5);
		expect(billedAt('system', 'gemini', changes)).toBe('gemini');
		expect(billedAt('microphone', 'gemini', changes)).toBe('gemini');
	});

	it('counts time on offline Whisper as free', () => {
		const changes = [
			{ track: 'system' as const, provider: null, atMs: HOUR / 4 },
			{ track: 'system' as const, provider: 'whisper' as const, atMs: HOUR / 4 + 1_000 }
		];
		const cost = ledgerCost(['microphone', 'system'], 'openai', changes, HOUR);
		expect(cost).toBeCloseTo(3.06 + 3.06 / 4);
		expect(billedAt('system', 'openai', changes)).toBe('whisper');
	});

	it('lists a run’s streams the way the rail counts them', () => {
		expect(runTracks(true, true, gemini)).toEqual(['microphone', 'system']);
		expect(runTracks(false, true, { ...gemini, secondTargetLanguage: 'de' })).toEqual([
			'system',
			'system:1'
		]);
	});
});
