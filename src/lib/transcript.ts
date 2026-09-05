import type { Origin, TranscriptLine } from './types';

export type TranscriptFormat = 'markdown' | 'text';

/** Consecutive lines from one source, bounded by pauses and a readable paragraph length. */
export interface TranscriptParagraph {
	/** Id of the paragraph's first line — a stable list key. */
	id: number;
	origin: Origin;
	text: string;
}

/** What the saved file calls each audio source, and what it calls itself. Passed in rather
 *  than hard-coded so the formatter stays free of Svelte, Tauri and the catalog while the
 *  document it writes still follows the interface language. */
export interface TranscriptLabels {
	title: string;
	origin: Record<Origin, string>;
	/** BCP 47 tag for the header's timestamp. */
	tag: string;
}

/** English, for tests and for any caller with nothing better to pass. */
export const DEFAULT_LABELS: TranscriptLabels = {
	title: 'Live captions transcript',
	origin: { microphone: 'Microphone', system: 'System' },
	tag: 'en-GB'
};

/**
 * Turn the newest-first log into chronological paragraphs. A source change, five-second
 * pause, timeline reset, or 600-character aggregation limit starts a new paragraph.
 * Individual caption lines are never split or truncated.
 */
export function groupTranscript(newestFirst: TranscriptLine[]): TranscriptParagraph[] {
	const paragraphs: TranscriptParagraph[] = [];
	let previousEnd: number | undefined;
	for (const line of [...newestFirst].reverse()) {
		const text = line.text.trim();
		if (!text) continue;
		const last = paragraphs[paragraphs.length - 1];
		const pause =
			line.startMs !== undefined &&
			previousEnd !== undefined &&
			(line.startMs - previousEnd >= 5000 || line.startMs < previousEnd);
		// Keep complete caption lines together. Long individual turns remain intact;
		// this bounds aggregation, never truncates the operator's text.
		if (last && last.origin === line.origin && !pause && last.text.length + text.length + 1 <= 600)
			last.text += ` ${text}`;
		else paragraphs.push({ id: line.id, origin: line.origin, text });
		previousEnd = line.endMs;
	}
	return paragraphs;
}

/** Produce a chronological, portable transcript without depending on Svelte or Tauri. */
export function formatTranscript(
	newestFirst: TranscriptLine[],
	format: TranscriptFormat,
	createdAt = new Date(),
	labels: TranscriptLabels = DEFAULT_LABELS
): string {
	const paragraphs = groupTranscript(newestFirst);

	if (format === 'text') {
		return paragraphs.map((p) => `${labels.origin[p.origin]}\n${p.text}\n`).join('\n');
	}

	const stamp = createdAt.toLocaleString(labels.tag);
	const header = `# ${labels.title}\n\n${stamp}\n`;
	const body = paragraphs.map((p) => `\n**${labels.origin[p.origin]}**\n\n${p.text}\n`).join('');
	return `${header}${body}`;
}

export function transcriptFilename(createdAt: Date, format: TranscriptFormat): string {
	const pad = (value: number) => String(value).padStart(2, '0');
	const stamp = `${createdAt.getFullYear()}${pad(createdAt.getMonth() + 1)}${pad(createdAt.getDate())}-${pad(createdAt.getHours())}${pad(createdAt.getMinutes())}${pad(createdAt.getSeconds())}`;
	return `transcript-${stamp}.${format === 'markdown' ? 'md' : 'txt'}`;
}
