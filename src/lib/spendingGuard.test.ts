// The spending guard's rules, as functions of times and amounts.

import { describe, expect, it } from 'vitest';
import {
	costWarningDue,
	durationWarningDue,
	IDLE_PAUSE_CHOICES,
	idlePauseDue,
	isBilled,
	lastSignOfLife,
	readChoice
} from './spendingGuard';

const MINUTE = 60_000;

describe('the idle pause', () => {
	it('is due once nothing has been captioned for the chosen minutes', () => {
		expect(idlePauseDue(10 * MINUTE, 1, 10)).toBe(false);
		expect(idlePauseDue(10 * MINUTE + 1, 1, 10)).toBe(true);
	});

	it('is never due while off, or before the session has given any sign of life', () => {
		expect(idlePauseDue(99 * MINUTE, 1, 0)).toBe(false);
		expect(idlePauseDue(99 * MINUTE, 0, 5)).toBe(false);
	});

	it('waits from the newest caption, or from the last Resume, whichever is later', () => {
		const captions = { system: { caption: 5_000 }, microphone: { caption: 9_000 } };
		expect(lastSignOfLife(captions, { system: 1_000 })).toBe(9_000);
		// Resumed after a long pause: the wait starts again from the Resume.
		expect(lastSignOfLife(captions, { system: 1_000, microphone: 50_000 })).toBe(50_000);
		expect(lastSignOfLife({ system: {} }, {})).toBe(0);
		expect(lastSignOfLife({}, {})).toBe(0);
	});
});

describe('the warnings', () => {
	it('say so once the estimate reaches the amount, and never while off', () => {
		expect(costWarningDue(4.99, 5)).toBe(false);
		expect(costWarningDue(5, 5)).toBe(true);
		expect(costWarningDue(500, 0)).toBe(false);
	});

	it('say so once the session reaches the hours, and never while off', () => {
		expect(durationWarningDue(2 * 3_600_000 - 1, 2)).toBe(false);
		expect(durationWarningDue(2 * 3_600_000, 2)).toBe(true);
		expect(durationWarningDue(99 * 3_600_000, 0)).toBe(false);
	});
});

describe('what the guard watches', () => {
	it('is the cloud engines: local Whisper and the demo cost nothing', () => {
		expect(isBilled('openai')).toBe(true);
		expect(isBilled('deepl')).toBe(true);
		expect(isBilled('whisper')).toBe(false);
		expect(isBilled('whisper-translate')).toBe(false);
		expect(isBilled('ondevice')).toBe(false);
	});

	it('reads a stored choice back only when it is one of the choices', () => {
		const storage = new Map<string, string>();
		globalThis.localStorage = {
			getItem: (key: string) => storage.get(key) ?? null
		} as Storage;
		storage.set('k', '15');
		expect(readChoice('k', IDLE_PAUSE_CHOICES)).toBe(15);
		storage.set('k', '7');
		expect(readChoice('k', IDLE_PAUSE_CHOICES)).toBe(0);
		storage.set('k', 'lots');
		expect(readChoice('k', IDLE_PAUSE_CHOICES)).toBe(0);
		Reflect.deleteProperty(globalThis, 'localStorage');
	});
});
