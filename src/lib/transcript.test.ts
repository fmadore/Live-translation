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
		expect(result.indexOf('Ça fonctionne.')).toBeLessThan(
			result.indexOf('Les sous-titres apparaissent sous.')
		);
		// The header keeps its export date — the transcript itself carries no clock times.
		// Asserted on the body only, since `toLocaleString` renders the header per locale.
		const body = result.slice(result.indexOf('**Microphone**'));
		expect(body).not.toMatch(/\d{1,2}[:.h]\d{2}/);
	});

	// Issue #23: the file is the part of this app that leaves the machine, so its headings and
	// its date follow the interface language rather than the formatter's defaults.
	it('writes its headings and its date in the language it is given', () => {
		const result = formatTranscript(lines, 'markdown', new Date('2026-09-21T10:00:00Z'), {
			title: 'Transcription des sous-titres',
			origin: { microphone: 'Microphone', system: 'Audio du système' },
			tag: 'fr-FR'
		});

		expect(result).toContain('# Transcription des sous-titres');
		expect(result).toContain('**Audio du système**');
		expect(result).toMatch(/21\/09\/2026/);
	});

	it('says the same thing in plain text', () => {
		const result = formatTranscript(lines, 'text', new Date(), {
			title: 'Transcription des sous-titres',
			origin: { microphone: 'Micro', system: 'Système' },
			tag: 'fr-FR'
		});

		expect(result).toContain(['Micro', 'Ça fonctionne.'].join('\n'));
		expect(result).toContain(['Système', 'Les sous-titres apparaissent sous.'].join('\n'));
	});
});

describe('transcriptFilename', () => {
	it('uses a filesystem-safe local timestamp and requested extension', () => {
		const date = new Date(2026, 8, 21, 9, 7, 5);
		expect(transcriptFilename(date, 'text')).toBe('transcript-20260921-090705.txt');
		expect(transcriptFilename(date, 'markdown')).toBe('transcript-20260921-090705.md');
	});
});
