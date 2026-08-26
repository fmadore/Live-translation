import { describe, expect, it } from 'vitest';
import {
	decodeRecovery,
	encodeRecovery,
	isDirty,
	newestLineId,
	NOTHING_SAVED,
	RECOVERY_VERSION,
	shouldGuardClose
} from './document';
import type { TranscriptLine } from './types';

function line(id: number, text = `line ${id}`): TranscriptLine {
	return { id, text, sourceText: `source ${id}`, origin: id % 2 ? 'microphone' : 'system' };
}

/** The store keeps the log newest-first; every helper here reads it that way. */
function log(count: number): TranscriptLine[] {
	return Array.from({ length: count }, (_, i) => line(count - i));
}

describe('saved / unsaved state', () => {
	it('treats an empty document as neither dirty nor saved', () => {
		expect(newestLineId([])).toBe(NOTHING_SAVED);
		expect(isDirty([], NOTHING_SAVED)).toBe(false);
	});

	it('is dirty until the newest line has been written', () => {
		const lines = log(3);
		expect(isDirty(lines, NOTHING_SAVED)).toBe(true);
		expect(isDirty(lines, newestLineId(lines))).toBe(false);
	});

	// Saving the same document twice is the ordinary case — an operator clicks Save Markdown
	// and then Save text, or just presses it again — and must not report unsaved work.
	it('stays saved across a duplicate save with nothing new in between', () => {
		const lines = log(3);
		const first = newestLineId(lines);
		expect(isDirty(lines, first)).toBe(false);
		expect(isDirty(lines, newestLineId(lines))).toBe(false);
	});

	it('goes dirty again as soon as one more line lands', () => {
		const lines = log(3);
		const saved = newestLineId(lines);
		expect(isDirty([line(4), ...lines], saved)).toBe(true);
	});

	// A long session is the case the old 1,000-line cap silently destroyed; the ids stay
	// comparable well past it.
	it('tracks a session far longer than the old truncation limit', () => {
		const lines = log(5000);
		expect(lines).toHaveLength(5000);
		expect(newestLineId(lines)).toBe(5000);
		expect(isDirty(lines, 4999)).toBe(true);
		expect(isDirty(lines, 5000)).toBe(false);
	});
});

describe('recovery snapshot', () => {
	it('round-trips the log with its metadata', () => {
		const lines = log(4);
		const at = new Date('2026-08-26T09:30:00.000Z');
		const snapshot = decodeRecovery(encodeRecovery(lines, at));

		expect(snapshot).not.toBeNull();
		expect(snapshot?.version).toBe(RECOVERY_VERSION);
		expect(snapshot?.savedAt).toBe('2026-08-26T09:30:00.000Z');
		expect(snapshot?.lines).toEqual(lines);
	});

	// The privacy promise in docs/privacy.md is only as good as what actually reaches the
	// file: finalized caption text, and nothing that identifies a provider or a device.
	it('writes caption fields and nothing else', () => {
		const contaminated = [
			{ ...line(1), apiKey: 'sk-live-should-never-be-here', device: 'Headset (Realtek)' }
		] as unknown as TranscriptLine[];
		const raw = encodeRecovery(contaminated, new Date('2026-08-26T09:30:00.000Z'));

		expect(raw).not.toContain('sk-live-should-never-be-here');
		expect(raw).not.toContain('Realtek');
		expect(Object.keys(JSON.parse(raw).lines[0]).sort()).toEqual([
			'id',
			'origin',
			'sourceText',
			'text'
		]);
	});

	it('refuses a spool that was truncated mid-write', () => {
		const raw = encodeRecovery(log(20), new Date());
		expect(decodeRecovery(raw.slice(0, Math.floor(raw.length / 2)))).toBeNull();
	});

	it('refuses junk, a foreign version, a bad timestamp, and an empty log', () => {
		expect(decodeRecovery('not json at all')).toBeNull();
		expect(decodeRecovery('null')).toBeNull();
		expect(decodeRecovery('[]')).toBeNull();
		expect(decodeRecovery(JSON.stringify({ version: 99, savedAt: '', lines: [line(1)] }))).toBeNull();
		expect(
			decodeRecovery(JSON.stringify({ version: RECOVERY_VERSION, savedAt: 'soon', lines: [line(1)] }))
		).toBeNull();
		expect(
			decodeRecovery(JSON.stringify({ version: RECOVERY_VERSION, savedAt: new Date().toISOString(), lines: [] }))
		).toBeNull();
	});

	it('drops individual lines that do not describe a caption', () => {
		const snapshot = decodeRecovery(
			JSON.stringify({
				version: RECOVERY_VERSION,
				savedAt: new Date('2026-08-26T09:30:00.000Z').toISOString(),
				lines: [line(2), { id: 'one', text: 1, origin: 'nowhere' }, line(1)]
			})
		);
		expect(snapshot?.lines).toEqual([line(2), line(1)]);
	});
});

describe('close guard', () => {
	it('leaves an idle, saved app alone', () => {
		expect(shouldGuardClose(false, false)).toBe(false);
	});

	it('guards unsaved text, and a running session even with nothing logged yet', () => {
		expect(shouldGuardClose(true, false)).toBe(true);
		expect(shouldGuardClose(false, true)).toBe(true);
		expect(shouldGuardClose(true, true)).toBe(true);
	});
});
