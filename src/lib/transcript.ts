import type { Lane, Origin, TargetLanguage, TranscriptLine, TranscriptMarker } from './types';

export type TranscriptFormat = 'markdown' | 'text' | 'vtt' | 'srt';

/** Older recovery files have no timing; never silently omit their text from an export. */
export function hasTranscriptTiming(lines: TranscriptLine[]): boolean {
	return (
		lines.length > 0 &&
		lines.every(
			(line) =>
				Number.isSafeInteger(line.startMs) &&
				Number.isSafeInteger(line.endMs) &&
				line.startMs! >= 0 &&
				line.endMs! >= line.startMs!
		)
	);
}

/**
 * The lines that reach into the last `windowMs` of the transcript's timeline, which ends
 * where its latest line does. A line that began earlier but is still running inside the
 * window is kept whole: a copied excerpt never starts mid-sentence. Lines without timing
 * cannot be placed, so they are all kept — callers offer the window only for timed logs.
 */
export function recentLines(lines: TranscriptLine[], windowMs: number): TranscriptLine[] {
	const latest = lines.reduce((end, line) => Math.max(end, line.endMs ?? -Infinity), -Infinity);
	if (latest === -Infinity) return lines;
	const from = latest - windowMs;
	return lines.filter((line) => line.endMs === undefined || line.endMs >= from);
}

/** The markers inside the same window as `recentLines`: those added at or after the start of
 *  the last `windowMs` of the lines' timeline. All of them when the lines have no timing. */
export function recentMarkers(
	lines: TranscriptLine[],
	markers: readonly TranscriptMarker[],
	windowMs: number
): TranscriptMarker[] {
	const latest = lines.reduce((end, line) => Math.max(end, line.endMs ?? -Infinity), -Infinity);
	if (latest === -Infinity) return [...markers];
	return markers.filter((marker) => marker.atMs >= latest - windowMs);
}

/** A marker's moment as a reader of the document reads it: `0:42:10`, from the start. */
export function markerTime(ms: number): string {
	const total = Math.max(0, Math.floor(ms / 1000));
	const pad = (n: number) => String(n).padStart(2, '0');
	return `${Math.floor(total / 3600)}:${pad(Math.floor(total / 60) % 60)}:${pad(total % 60)}`;
}

/** What a marker says: the operator's text, or a numbered section for a blank one. `number`
 *  is its place among the document's markers, from 1. */
export function markerTitle(
	marker: TranscriptMarker,
	number: number,
	section: (n: number) => string = (n) => `Section ${n}`
): string {
	return marker.label || section(number);
}

/** The paragraphs with the markers between them, in document order (by id: lines and markers
 *  share one counter), each marker with its number. `groupTranscript` given the same markers
 *  has already broken every paragraph a marker falls inside. */
export type TranscriptItem =
	| { kind: 'paragraph'; paragraph: TranscriptParagraph }
	| { kind: 'marker'; marker: TranscriptMarker; number: number };

export function withMarkers(
	paragraphs: TranscriptParagraph[],
	markers: readonly TranscriptMarker[]
): TranscriptItem[] {
	if (!markers.length) return paragraphs.map((paragraph) => ({ kind: 'paragraph', paragraph }));
	const ordered = [...markers].sort((a, b) => a.id - b.id);
	const items: TranscriptItem[] = [];
	let next = 0;
	for (const paragraph of paragraphs) {
		while (next < ordered.length && ordered[next].id < paragraph.id) {
			items.push({ kind: 'marker', marker: ordered[next], number: next + 1 });
			next++;
		}
		items.push({ kind: 'paragraph', paragraph });
	}
	for (; next < ordered.length; next++)
		items.push({ kind: 'marker', marker: ordered[next], number: next + 1 });
	return items;
}

function cueTime(ms: number, separator: string): string {
	const hours = Math.floor(ms / 3600000);
	const minutes = Math.floor(ms / 60000) % 60;
	const seconds = Math.floor(ms / 1000) % 60;
	return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}${separator}${String(ms % 1000).padStart(3, '0')}`;
}

/** WebVTT cue text is markup: `&`, `<` and `>` are written as entities. */
function escapeVtt(value: string): string {
	return value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

/** SRT has no entities, so text is written as it is (review B9: "Q&A" came out `Q&amp;A`).
 *  Only a literal `-->`, which a reader takes for a timing line, is broken up. */
function escapeSrt(value: string): string {
	return value.replace(/-->/g, '->');
}

function timedTranscript(
	lines: TranscriptLine[],
	format: 'vtt' | 'srt',
	labels: TranscriptLabels,
	original: boolean,
	markers: readonly TranscriptMarker[]
): string {
	if (!lines.length) return format === 'vtt' ? 'WEBVTT\n\n' : '';
	if (!hasTranscriptTiming(lines)) throw new Error('Transcript has no valid cue timing');
	// Completion order may differ from start order when both sources speak together.
	const ordered = [...lines].sort((a, b) => a.startMs! - b.startMs! || a.id - b.id);
	const cues = ordered
		.filter((line) => line.text.trim())
		.map((line, i) => {
			const separator = format === 'srt' ? ',' : '.';
			const start = line.startMs!;
			// A provider can finalize a whole turn in one frame. Give that cue a minimal interval.
			const end = Math.max(start + 1, line.endMs!);
			const escape = format === 'vtt' ? escapeVtt : escapeSrt;
			// One line per part: a line break, a blank one above all, would end the cue.
			const oneLine = (value: string) => escape(value.trim().replace(/\s*\r?\n\s*/g, ' '));
			const text = oneLine(line.text);
			const origin = escape(speakerLabel(labels, line));
			const payload = format === 'vtt' ? `<v ${origin}>${text}</v>` : `[${origin}] ${text}`;
			// The original speech as the cue's second line, in italics, the way bilingual
			// subtitles are usually set. Both players and editors take `<i>` in either format.
			const source = oneLine(originalOf(line, original));
			const second = source ? `\n<i>${source}</i>` : '';
			return {
				at: start,
				text: `${i + 1}\n${cueTime(start, separator)} --> ${cueTime(end, separator)}\n${payload}${second}\n`
			};
		});
	// SRT has no comments, and a marker is not speech: it gets no cue there. WebVTT's NOTE
	// block is a comment players skip, so a marker is kept as one, before the first cue at or
	// after its moment. Never `-->` inside one, which would end the comment.
	const notes =
		format === 'vtt'
			? [...markers]
					.sort((a, b) => a.id - b.id)
					.map((marker, i) => ({
						at: marker.atMs,
						text: `NOTE ${markerTime(marker.atMs)} ${markerTitle(marker, i + 1, labels.section).replace(/-->/g, '->')}\n`
					}))
					.sort((a, b) => a.at - b.at)
			: [];
	const blocks: string[] = [];
	let next = 0;
	for (const cue of cues) {
		while (next < notes.length && notes[next].at <= cue.at) blocks.push(notes[next++].text);
		blocks.push(cue.text);
	}
	while (next < notes.length) blocks.push(notes[next++].text);
	return (format === 'vtt' ? 'WEBVTT\n\n' : '') + blocks.join('\n');
}

/** Consecutive lines from one source, bounded by pauses and a readable paragraph length. */
export interface TranscriptParagraph {
	/** Id of the paragraph's first line — a stable list key. */
	id: number;
	origin: Origin;
	text: string;
	/** What was said before translation, joined the same way. Empty for subtitles. */
	sourceText: string;
	/** The caption language, for a session that captioned in two; absent otherwise. */
	language?: TargetLanguage;
	/** Which of the session's languages: the second never repeats the original speech. */
	lane: Lane;
}

/** What the saved file calls each audio source, and what it calls itself. Passed in rather
 *  than hard-coded so the formatter stays free of Svelte, Tauri and the catalog while the
 *  document it writes still follows the interface language. */
export interface TranscriptLabels {
	title: string;
	origin: Record<Origin, string>;
	/** What introduces the original speech in a bilingual text or Markdown export. */
	original: string;
	/** A caption language's name, for a transcript in two of them. */
	language?: (code: TargetLanguage) => string;
	/** BCP 47 tag for the header's timestamp. */
	tag: string;
	/** What a marker left blank is called, by its number: "Section 2". English when absent. */
	section?: (n: number) => string;
}

/** English, for tests and for any caller with nothing better to pass. */
export const DEFAULT_LABELS: TranscriptLabels = {
	title: 'Live captions transcript',
	origin: { microphone: 'Microphone', system: 'System' },
	original: 'Original',
	tag: 'en-GB',
	section: (n) => `Section ${n}`
};

export interface TranscriptOptions {
	/** Write the original speech under each translation. Lines without any are unchanged. */
	original?: boolean;
	/** The operator's markers: a heading in text and Markdown, a NOTE in WebVTT, nothing in
	 *  SRT. */
	markers?: readonly TranscriptMarker[];
}

/** Whether any line carries original speech, which is what makes a bilingual export
 *  possible: translation keeps it, same-language subtitles have none. */
export function hasOriginalSpeech(lines: TranscriptLine[]): boolean {
	return lines.some((line) => line.sourceText.trim() !== '');
}

/** A source's label, with its caption language when the line records one. */
function speakerLabel(
	labels: TranscriptLabels,
	line: { origin: Origin; language?: TargetLanguage }
): string {
	const origin = labels.origin[line.origin];
	if (!line.language) return origin;
	return `${origin} · ${labels.language?.(line.language) ?? line.language.toUpperCase()}`;
}

/** The original speech worth writing beside a line: only when asked, and only once — the
 *  second language heard the same speech as the first. */
function originalOf(line: { sourceText: string; lane?: Lane }, original: boolean): string {
	return original && (line.lane ?? 0) === 0 ? line.sourceText.trim() : '';
}

/**
 * Turn the newest-first log into chronological paragraphs. A source change, five-second
 * pause, timeline reset, or 600-character aggregation limit starts a new paragraph.
 * Individual caption lines are never split or truncated.
 *
 * A session that captioned in two languages has two independent streams of lines, which
 * finish turns at their own moments. Each is grouped on its own — interleaved, they would
 * break one another's paragraphs every few seconds — and the paragraphs are then put back in
 * the order they began.
 */
export function groupTranscript(
	newestFirst: TranscriptLine[],
	markers: readonly TranscriptMarker[] = []
): TranscriptParagraph[] {
	const streams = new Map<Lane, TranscriptLine[]>();
	for (const line of [...newestFirst].reverse()) {
		const lane = line.lane ?? 0;
		const stream = streams.get(lane);
		if (stream) stream.push(line);
		else streams.set(lane, [line]);
	}
	// A marker ends the paragraph it falls in, in every stream: what follows it is the next talk.
	const breaks = markers.map((marker) => marker.id).sort((a, b) => a - b);
	const paragraphs = [...streams.entries()].flatMap(([lane, lines]) =>
		groupStream(lines, lane, breaks)
	);
	return streams.size > 1 ? paragraphs.sort((a, b) => a.id - b.id) : paragraphs;
}

function groupStream(
	chronological: TranscriptLine[],
	lane: Lane,
	breaks: readonly number[]
): TranscriptParagraph[] {
	const paragraphs: TranscriptParagraph[] = [];
	let previousEnd: number | undefined;
	let previousId = -Infinity;
	let nextBreak = 0;
	for (const line of chronological) {
		const text = line.text.trim();
		if (!text) continue;
		const sourceText = line.sourceText.trim();
		const last = paragraphs[paragraphs.length - 1];
		// Lines come in id order, so the breaks are passed through once.
		let marked = false;
		while (nextBreak < breaks.length && breaks[nextBreak] < line.id) {
			if (breaks[nextBreak] > previousId) marked = true;
			nextBreak++;
		}
		previousId = line.id;
		const pause =
			marked ||
			(line.startMs !== undefined &&
				previousEnd !== undefined &&
				(line.startMs - previousEnd >= 5000 || line.startMs < previousEnd));
		// Keep complete caption lines together. Long individual turns remain intact;
		// this bounds aggregation, never truncates the operator's text.
		if (
			last &&
			last.origin === line.origin &&
			last.language === line.language &&
			!pause &&
			last.text.length + text.length + 1 <= 600
		) {
			last.text += ` ${text}`;
			if (sourceText)
				last.sourceText = last.sourceText ? `${last.sourceText} ${sourceText}` : sourceText;
		} else
			paragraphs.push({
				id: line.id,
				origin: line.origin,
				text,
				sourceText,
				lane,
				...(line.language ? { language: line.language } : {})
			});
		previousEnd = line.endMs;
	}
	return paragraphs;
}

/** Paragraphs in sections by caption language, in the order each language first appears.
 *  A transcript in one language is one section with no heading, as it always was. */
function sections(paragraphs: TranscriptParagraph[]) {
	const byLanguage = new Map<TargetLanguage | undefined, TranscriptParagraph[]>();
	for (const p of paragraphs) {
		const section = byLanguage.get(p.language);
		if (section) section.push(p);
		else byLanguage.set(p.language, [p]);
	}
	return [...byLanguage.entries()].map(([language, list]) => ({ language, paragraphs: list }));
}

/** Produce a chronological, portable transcript without depending on Svelte or Tauri. */
export function formatTranscript(
	newestFirst: TranscriptLine[],
	format: TranscriptFormat,
	createdAt = new Date(),
	labels: TranscriptLabels = DEFAULT_LABELS,
	{ original = false, markers = [] }: TranscriptOptions = {}
): string {
	if (format === 'srt' || format === 'vtt')
		return timedTranscript(newestFirst, format, labels, original, markers);
	const parts = sections(groupTranscript(newestFirst, markers));
	// A document of markers alone still writes them.
	if (!parts.length && markers.length) parts.push({ language: undefined, paragraphs: [] });
	const languageName = (language: TargetLanguage) =>
		labels.language?.(language) ?? language.toUpperCase();
	// Within a section the language is its heading, so paragraphs name only their source.
	const label = (p: TranscriptParagraph) => labels.origin[p.origin];
	// Every section shows every marker, so a document in two languages has each talk's start in
	// both; the numbers are the document's, so a blank marker has the same one in both.
	const title = (marker: TranscriptMarker, number: number) =>
		`${markerTitle(marker, number, labels.section)} · ${markerTime(marker.atMs)}`;

	if (format === 'text') {
		return parts
			.map(({ language, paragraphs }) => {
				const heading = language ? `${languageName(language).toUpperCase()}\n\n` : '';
				const body = withMarkers(paragraphs, markers)
					.map((item) => {
						if (item.kind === 'marker') return `--- ${title(item.marker, item.number)} ---\n`;
						const p = item.paragraph;
						const source = originalOf(p, original);
						return `${label(p)}\n${p.text}\n` + (source ? `${labels.original}: ${source}\n` : '');
					})
					.join('\n');
				return heading + body;
			})
			.join('\n');
	}

	const stamp = createdAt.toLocaleString(labels.tag);
	const header = `# ${labels.title}\n\n${stamp}\n`;
	const body = parts
		.map(({ language, paragraphs }) => {
			const heading = language ? `\n## ${languageName(language)}\n` : '';
			// One level under the language's heading, when there is one.
			const level = language ? '###' : '##';
			return (
				heading +
				withMarkers(paragraphs, markers)
					.map((item) => {
						if (item.kind === 'marker') return `\n${level} ${title(item.marker, item.number)}\n`;
						const p = item.paragraph;
						const source = originalOf(p, original);
						return (
							`\n**${label(p)}**\n\n${p.text}\n` +
							(source ? `\n> *${labels.original}:* ${source}\n` : '')
						);
					})
					.join('')
			);
		})
		.join('');
	return `${header}${body}`;
}

export function transcriptFilename(createdAt: Date, format: TranscriptFormat): string {
	const pad = (value: number) => String(value).padStart(2, '0');
	const stamp = `${createdAt.getFullYear()}${pad(createdAt.getMonth() + 1)}${pad(createdAt.getDate())}-${pad(createdAt.getHours())}${pad(createdAt.getMinutes())}${pad(createdAt.getSeconds())}`;
	const extension = { markdown: 'md', text: 'txt', vtt: 'vtt', srt: 'srt' }[format];
	return `transcript-${stamp}.${extension}`;
}
