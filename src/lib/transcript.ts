import type { Origin, TranscriptLine } from './types';

export type TranscriptFormat = 'markdown' | 'text';

/** A run of consecutive lines from the same audio source, rendered as one paragraph. */
export interface TranscriptParagraph {
	/** Id of the paragraph's first line — a stable list key. */
	id: number;
	origin: Origin;
	text: string;
}

/** Human label for an audio source; replaces per-line timestamps in the transcript. */
export const ORIGIN_LABEL: Record<Origin, string> = {
	microphone: 'Microphone',
	system: 'System'
};

/**
 * Turn the newest-first log into chronological paragraphs, one per contiguous run of the
 * same source. A speaker change (mic ⇄ system) starts a new paragraph; everything else is
 * joined into flowing text.
 */
export function groupTranscript(newestFirst: TranscriptLine[]): TranscriptParagraph[] {
	const paragraphs: TranscriptParagraph[] = [];
	for (const line of [...newestFirst].reverse()) {
		const text = line.text.trim();
		if (!text) continue;
		const last = paragraphs[paragraphs.length - 1];
		if (last && last.origin === line.origin) last.text += ` ${text}`;
		else paragraphs.push({ id: line.id, origin: line.origin, text });
	}
	return paragraphs;
}

/** Produce a chronological, portable transcript without depending on Svelte or Tauri. */
export function formatTranscript(
	newestFirst: TranscriptLine[],
	format: TranscriptFormat,
	createdAt = new Date()
): string {
	const paragraphs = groupTranscript(newestFirst);

	if (format === 'text') {
		return paragraphs
			.map((p) => `${ORIGIN_LABEL[p.origin]}\n${p.text}\n`)
			.join('\n');
	}

	const header = `# Live captions transcript\n\n${createdAt.toLocaleString()}\n`;
	const body = paragraphs
		.map((p) => `\n**${ORIGIN_LABEL[p.origin]}**\n\n${p.text}\n`)
		.join('');
	return `${header}${body}`;
}

export function transcriptFilename(createdAt: Date, format: TranscriptFormat): string {
	const pad = (value: number) => String(value).padStart(2, '0');
	const stamp = `${createdAt.getFullYear()}${pad(createdAt.getMonth() + 1)}${pad(createdAt.getDate())}-${pad(createdAt.getHours())}${pad(createdAt.getMinutes())}${pad(createdAt.getSeconds())}`;
	return `transcript-${stamp}.${format === 'markdown' ? 'md' : 'txt'}`;
}
