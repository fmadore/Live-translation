import type { TranscriptLine } from './types';

export type TranscriptFormat = 'markdown' | 'text';

/** Produce a chronological, portable transcript without depending on Svelte or Tauri. */
export function formatTranscript(
	newestFirst: TranscriptLine[],
	format: TranscriptFormat,
	createdAt = new Date()
): string {
	const lines = [...newestFirst].reverse();
	if (format === 'text') {
		return `${lines
			.map((line) => {
				const source = line.sourceText ? `\n  Source: ${line.sourceText}` : '';
				return `[${line.time}] ${line.origin}: ${line.text}${source}`;
			})
			.join('\n\n')}\n`;
	}

	const header = `# Live captions transcript\n\n${createdAt.toLocaleString()} · STIAS DH & AI workshop\n\n`;
	const body = lines
		.map((line) => {
			const source = line.sourceText ? `\n  - _source_: ${line.sourceText}` : '';
			return `- **${line.time}** · ${line.origin}\n  - ${line.text}${source}`;
		})
		.join('\n');
	return `${header}${body}\n`;
}

export function transcriptFilename(createdAt: Date, format: TranscriptFormat): string {
	const pad = (value: number) => String(value).padStart(2, '0');
	const stamp = `${createdAt.getFullYear()}${pad(createdAt.getMonth() + 1)}${pad(createdAt.getDate())}-${pad(createdAt.getHours())}${pad(createdAt.getMinutes())}${pad(createdAt.getSeconds())}`;
	return `transcript-${stamp}.${format === 'markdown' ? 'md' : 'txt'}`;
}
