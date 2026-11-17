// The operator's markers in the transcript (a talk starting, a speaker's name): how they divide
// the log, what each export makes of them, and how they survive the recovery spool and the
// history log — including files written before markers existed.

import { describe, expect, it } from 'vitest';
import {
	cleanMarkerLabel,
	decodeRecovery,
	encodeRecovery,
	isDirty,
	MARKER_LABEL_MAX,
	newestDocumentId,
	readMarker
} from './document';
import { decodeSession, encodeSessionAppend, encodeSessionLog, type SavedSession } from './history';
import { matchesSession } from './historySearch';
import {
	DEFAULT_LABELS,
	formatTranscript,
	groupTranscript,
	markerTime,
	markerTitle,
	recentMarkers,
	withMarkers
} from './transcript';
import type { TranscriptLine, TranscriptMarker } from './types';

const line = (id: number, text: string, startMs: number, lane?: 1): TranscriptLine => ({
	id,
	text,
	sourceText: '',
	origin: 'system',
	...(lane ? { lane } : {}),
	startMs,
	endMs: startMs + 900
});
const marker = (id: number, atMs: number, label = ''): TranscriptMarker => ({ id, atMs, label });

/** Newest first, as the store holds them: two lines, a marker, two lines, all one speaker
 *  within a few seconds, which would otherwise be one paragraph. */
const lines = [
	line(5, 'Fifth.', 4000),
	line(4, 'Fourth.', 3000),
	line(2, 'Second.', 1000),
	line(1, 'First.', 0)
];
const talk = marker(3, 2500, 'Talk 2 starts');

describe('markers in the log', () => {
	it('end the paragraph they fall in, and sit between the two halves', () => {
		const paragraphs = groupTranscript(lines, [talk]);
		expect(paragraphs.map((p) => p.text)).toEqual(['First. Second.', 'Fourth. Fifth.']);
		const items = withMarkers(paragraphs, [talk]);
		expect(items.map((item) => item.kind)).toEqual(['paragraph', 'marker', 'paragraph']);
		expect(items[1]).toMatchObject({ kind: 'marker', number: 1, marker: talk });
	});

	it('leave the paragraphs as they were without markers', () => {
		expect(groupTranscript(lines).map((p) => p.text)).toEqual(['First. Second. Fourth. Fifth.']);
		expect(withMarkers(groupTranscript(lines), []).every((i) => i.kind === 'paragraph')).toBe(true);
	});

	it('break both caption languages, and number by the order they were added', () => {
		const both = [line(6, 'Zweite.', 3000, 1), line(5, 'Second.', 3000), ...lines.slice(2)];
		const markers = [marker(4, 2000), marker(7, 9000, 'End')];
		const items = withMarkers(groupTranscript(both, markers), markers);
		expect(items.map((i) => (i.kind === 'marker' ? `#${i.number}` : i.paragraph.text))).toEqual([
			'First. Second.',
			'#1',
			'Second.',
			'Zweite.',
			'#2'
		]);
	});

	it('are titled by their text, or numbered when blank, and timed from the start', () => {
		expect(markerTitle(talk, 1)).toBe('Talk 2 starts');
		expect(markerTitle(marker(1, 0), 2)).toBe('Section 2');
		expect(markerTitle(marker(1, 0), 3, (n) => `Abschnitt ${n}`)).toBe('Abschnitt 3');
		expect(markerTime(2_530_400)).toBe('0:42:10');
		expect(markerTime(3_725_000)).toBe('1:02:05');
		expect(markerTime(-5)).toBe('0:00:00');
	});

	it('come with a copied excerpt when they are inside its window', () => {
		const late = marker(6, 4500, 'Questions');
		expect(recentMarkers(lines, [talk, late], 1000)).toEqual([late]);
		expect(recentMarkers(lines, [talk, late], 60_000)).toEqual([talk, late]);
		const untimed = [{ ...lines[0], startMs: undefined, endMs: undefined }];
		expect(recentMarkers(untimed, [talk], 1000)).toEqual([talk]);
	});

	it('make the document newer, and so unsaved', () => {
		expect(newestDocumentId(lines)).toBe(5);
		expect(newestDocumentId(lines, [marker(9, 0)])).toBe(9);
		expect(isDirty(lines, 5)).toBe(false);
		expect(isDirty(lines, 5, [marker(6, 5000)])).toBe(true);
	});
});

describe('markers in an export', () => {
	const at = new Date('2026-11-16T09:30:00Z');

	it('are a separator line in plain text', () => {
		const text = formatTranscript(lines, 'text', at, DEFAULT_LABELS, { markers: [talk] });
		expect(text).toBe(
			'System\nFirst. Second.\n\n--- Talk 2 starts · 0:00:02 ---\n\nSystem\nFourth. Fifth.\n'
		);
	});

	it('are a heading in Markdown, one level under a language heading', () => {
		const markdown = formatTranscript(lines, 'markdown', at, DEFAULT_LABELS, {
			markers: [marker(3, 2500)]
		});
		expect(markdown).toContain('\n## Section 1 · 0:00:02\n');
		const bilingual = [
			{ ...line(6, 'Zweite.', 3000, 1), language: 'de' as const },
			{ ...line(5, 'Second.', 3000), language: 'en' as const },
			{ ...line(1, 'First.', 0), language: 'en' as const }
		];
		const both = formatTranscript(bilingual, 'markdown', at, DEFAULT_LABELS, {
			markers: [marker(2, 2000, 'Break')]
		});
		// Each language's section has the marker, under its own heading.
		expect(both.match(/\n### Break · 0:00:02\n/g)).toHaveLength(2);
	});

	it('are written even when they are all there is', () => {
		expect(formatTranscript([], 'text', at, DEFAULT_LABELS, { markers: [talk] })).toBe(
			'--- Talk 2 starts · 0:00:02 ---\n'
		);
	});

	it('are a NOTE in WebVTT, before the first cue at or after their moment', () => {
		const vtt = formatTranscript(lines, 'vtt', at, DEFAULT_LABELS, {
			markers: [marker(3, 2500, 'Q --> A'), marker(9, 60_000)]
		});
		const blocks = vtt.split('\n\n');
		expect(blocks[0]).toBe('WEBVTT');
		expect(blocks[3]).toBe('NOTE 0:00:02 Q -> A');
		expect(blocks[4]).toMatch(/^3\n00:00:03.000 --> /);
		expect(blocks.at(-1)).toBe('NOTE 0:01:00 Section 2\n');
		// The cues keep their numbers: a note is not a cue.
		expect(vtt).toContain('\n4\n00:00:04.000');
	});

	it('are left out of SRT, which has no comments: every block stays a cue', () => {
		const srt = formatTranscript(lines, 'srt', at, DEFAULT_LABELS, { markers: [talk] });
		expect(srt).not.toContain('Talk 2');
		expect(srt).toBe(formatTranscript(lines, 'srt', at, DEFAULT_LABELS));
	});
});

describe('marker labels and records', () => {
	it('are one line, without control characters, and bounded', () => {
		expect(cleanMarkerLabel('  Talk\n2\u0007 starts  ')).toBe('Talk 2 starts');
		expect(cleanMarkerLabel(42)).toBe('');
		const long = 'é'.repeat(MARKER_LABEL_MAX + 10);
		expect(Array.from(cleanMarkerLabel(long))).toHaveLength(MARKER_LABEL_MAX);
		// A cut never splits a character that takes two code units.
		expect(cleanMarkerLabel('😀'.repeat(MARKER_LABEL_MAX + 1))).toBe('😀'.repeat(MARKER_LABEL_MAX));
	});

	it('are read warily from a file', () => {
		expect(readMarker({ id: 3, atMs: 2500.4, label: ' Talk\n2 ' })).toEqual({
			id: 3,
			atMs: 2500,
			label: 'Talk 2'
		});
		for (const bad of [
			null,
			'marker',
			{ id: 0, atMs: 0, label: '' },
			{ id: 1.5, atMs: 0, label: '' },
			{ id: 1, atMs: -1, label: '' },
			{ id: 1, atMs: Infinity, label: '' },
			{ id: 1, atMs: 0 },
			{ atMs: 0, label: '' }
		])
			expect(readMarker(bad)).toBeNull();
	});
});

describe('markers in the recovery spool', () => {
	const savedAt = new Date('2026-11-16T09:30:00.000Z');

	it('round-trip with the lines', () => {
		const snapshot = decodeRecovery(encodeRecovery(lines, savedAt, [talk]));
		expect(snapshot?.markers).toEqual([talk]);
		expect(snapshot?.lines).toHaveLength(4);
	});

	it('add nothing to a spool without them, which reads as it always did', () => {
		const raw = encodeRecovery(lines, savedAt);
		expect(JSON.parse(raw)).not.toHaveProperty('markers');
		expect(decodeRecovery(raw)).not.toHaveProperty('markers');
	});

	it('cost nothing but themselves when damaged', () => {
		const raw = JSON.parse(encodeRecovery(lines, savedAt, [talk])) as Record<string, unknown>;
		raw.markers = [{ id: 'x' }, talk];
		expect(decodeRecovery(JSON.stringify(raw))?.markers).toEqual([talk]);
		raw.markers = 'not a list';
		const snapshot = decodeRecovery(JSON.stringify(raw));
		expect(snapshot?.lines).toHaveLength(4);
		expect(snapshot).not.toHaveProperty('markers');
	});
});

describe('markers in the history log', () => {
	const id = '3f6c1a2e-8b4d-4c1e-9a7f-2d5e6b8c9a01';
	const session: SavedSession = {
		version: 1,
		id,
		startedAt: '2026-11-16T09:00:00.000Z',
		savedAt: '2026-11-16T09:10:00.000Z',
		endedAt: null,
		durationMs: 600_000,
		mode: 'transcribe',
		sourceLanguage: 'auto',
		targetLanguage: null,
		lines,
		markers: [marker(6, 5000, 'Questions'), talk]
	};

	it('are records of their own, read back newest first', () => {
		const raw = encodeSessionLog(session);
		expect(raw).toContain(`{"marker":{"id":3,"atMs":2500,"label":"Talk 2 starts"}}`);
		expect(decodeSession(raw, id)?.markers).toEqual(session.markers);
	});

	it('are appended once each, after what is already on disk', () => {
		const first = encodeSessionLog({ ...session, markers: [talk] });
		const more = encodeSessionAppend(session, lines.length, 1);
		expect(more.match(/"marker"/g)).toHaveLength(1);
		expect(more).toContain('Questions');
		expect(decodeSession(first + more, id)?.markers).toEqual(session.markers);
		// Called as before markers, an append still writes the lines and their progress.
		expect(encodeSessionAppend({ ...session, markers: undefined }, 2)).not.toContain('marker');
	});

	it('leave a log without them reading as it did, with no markers field', () => {
		const raw = encodeSessionLog({ ...session, markers: undefined });
		expect(raw).not.toContain('marker');
		expect(decodeSession(raw, id)).not.toHaveProperty('markers');
	});

	it('are skipped when damaged, without costing the session', () => {
		const raw = encodeSessionLog({ ...session, markers: [] }) + '{"marker":{"id":-1}}\n';
		const read = decodeSession(raw, id);
		expect(read?.lines).toHaveLength(4);
		expect(read).not.toHaveProperty('markers');
	});

	it('are found by the History search', () => {
		const filter = { query: 'questions', from: '', to: '', language: '' };
		expect(matchesSession(session, filter)).toBe(true);
		expect(matchesSession({ ...session, markers: undefined }, filter)).toBe(false);
	});
});
