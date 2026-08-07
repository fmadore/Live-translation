import { describe, expect, it } from 'vitest';
import { formatTranscript, groupTranscript, transcriptFilename } from './transcript';
import type { TranscriptLine } from './types';

/** The store keeps the log newest-first, so fixtures are written that way too. */
const lines: TranscriptLine[] = [
	{ id: 4, text: 'problématique.', sourceText: '', origin: 'microphone' },
	{ id: 3, text: 'Les fenêtres, ceci est un peu', sourceText: '', origin: 'microphone' },
	{ id: 2, text: 'Les sous-titres apparaissent sous.', sourceText: '', origin: 'system' },
	{ id: 1, text: 'Ça fonctionne.', sourceText: '', origin: 'microphone' }
];

describe('groupTranscript', () => {
	it('orders lines chronologically and starts a paragraph on each source change', () => {
		expect(groupTranscript(lines)).toEqual([
			{ id: 1, origin: 'microphone', text: 'Ça fonctionne.' },
			{ id: 2, origin: 'system', text: 'Les sous-titres apparaissent sous.' },
			{ id: 3, origin: 'microphone', text: 'Les fenêtres, ceci est un peu problématique.' }
		]);
	});

	it('drops blank lines', () => {
		const withBlank: TranscriptLine[] = [
			{ id: 2, text: '   ', sourceText: '', origin: 'system' },
			{ id: 1, text: 'Bonjour', sourceText: '', origin: 'system' }
		];
		expect(groupTranscript(withBlank)).toEqual([{ id: 1, origin: 'system', text: 'Bonjour' }]);
	});
});

describe('formatTranscript', () => {
	it('exports plain text as labelled paragraphs without timestamps', () => {
		expect(formatTranscript(lines, 'text')).toBe(
			'Microphone\nÇa fonctionne.\n' +
				'\nSystem\nLes sous-titres apparaissent sous.\n' +
				'\nMicrophone\nLes fenêtres, ceci est un peu problématique.\n'
		);
	});

	it('exports Markdown with a stable heading and bold source labels', () => {
		const result = formatTranscript(lines, 'markdown', new Date('2026-09-21T10:00:00Z'));
		expect(result).toContain('# Live captions transcript');
		expect(result).toContain('**Microphone**');
		expect(result).toContain('**System**');
		expect(result).not.toMatch(/\d{2}:\d{2}:\d{2}/);
		expect(result.indexOf('Ça fonctionne.')).toBeLessThan(
			result.indexOf('Les sous-titres apparaissent sous.')
		);
	});
});

describe('transcriptFilename', () => {
	it('uses a filesystem-safe local timestamp and requested extension', () => {
		const date = new Date(2026, 8, 21, 9, 7, 5);
		expect(transcriptFilename(date, 'text')).toBe('transcript-20260921-090705.txt');
		expect(transcriptFilename(date, 'markdown')).toBe('transcript-20260921-090705.md');
	});
});
