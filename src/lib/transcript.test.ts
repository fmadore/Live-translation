import { describe, expect, it } from 'vitest';
import { formatTranscript, transcriptFilename } from './transcript';
import type { TranscriptLine } from './types';

const lines: TranscriptLine[] = [
	{
		id: 2,
		time: '10:00:02',
		text: 'Bonjour',
		sourceText: 'Hello',
		origin: 'microphone'
	},
	{
		id: 1,
		time: '10:00:01',
		text: 'Original subtitle',
		sourceText: '',
		origin: 'system'
	}
];

describe('formatTranscript', () => {
	it('exports plain text chronologically and keeps source text optional', () => {
		expect(formatTranscript(lines, 'text')).toBe(
			'[10:00:01] system: Original subtitle\n\n[10:00:02] microphone: Bonjour\n  Source: Hello\n'
		);
	});

	it('exports Markdown with a stable heading', () => {
		const result = formatTranscript(lines, 'markdown', new Date('2026-09-21T10:00:00Z'));
		expect(result).toContain('# Live captions transcript');
		expect(result.indexOf('Original subtitle')).toBeLessThan(result.indexOf('Bonjour'));
		expect(result).toContain('_source_: Hello');
	});
});

describe('transcriptFilename', () => {
	it('uses a filesystem-safe local timestamp and requested extension', () => {
		const date = new Date(2026, 8, 21, 9, 7, 5);
		expect(transcriptFilename(date, 'text')).toBe('transcript-20260921-090705.txt');
		expect(transcriptFilename(date, 'markdown')).toBe('transcript-20260921-090705.md');
	});
});
