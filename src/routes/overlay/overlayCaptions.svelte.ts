// What the overlay shows: one current turn per origin, the finished turn before it, and that
// origin's reading context, plus when each of them expires. Kept apart from the window so it
// can be exercised without fonts, a ResizeObserver or a Tauri window.

import { appendCaptionHistory, type CaptionLayout } from '$lib/captionLayout';
import { cleanSpeech } from '$lib/cleanSpeech';
import { createCaptionPresenter, holdSeconds, type CaptionPace } from '$lib/reading';
import { captionBudget, clampOverlayWidth } from '$lib/types';
import type { Caption, Origin, StatusUpdate } from '$lib/types';

/** Stable render order: the remote speaker (system) above the room mic. */
export const ORIGIN_ORDER: readonly Origin[] = ['system', 'microphone'];

/** An in-progress line that stalls (no turn-complete arriving) clears after this. */
const INTERIM_HOLD_MS = 3000;

/** A speaker's previous turn and current turn are usually one continuous sentence, so in
 *  Compact they render as one block: the tail of the finished turn (dimmed) flowing into the
 *  live one, sharing one character budget. Only the room left by the current turn goes to the
 *  lead-in, and only when it is worth reading: a readable fragment of a previous sentence is
 *  the same amount of text in a narrow region as in a wide one. */
const MIN_LEAD_CHARS = 40;

/** Keep the last `limit` characters, cutting on a word boundary. */
export function tail(text: string, limit: number): string {
	const t = text.replace(/\s+/g, ' ').trim();
	if (t.length <= limit) return t;
	let cut = t.length - limit;
	// Don't start mid-word: jump to the next space if it's close.
	const sp = t.indexOf(' ', cut);
	if (sp !== -1 && sp - cut < 24) cut = sp + 1;
	return '… ' + t.slice(cut);
}

export interface CaptionLine {
	origin: Origin;
	/** Earlier context, drawn dimmed ahead of the turn. */
	lead: string;
	text: string;
	interim: boolean;
}

export interface ReadingSettings {
	layout: CaptionLayout;
	hideFillers: boolean;
	/** Seconds a finished Fit/Compact caption stays up. */
	hold: number;
	pace: CaptionPace;
	/** Compact line measure, in `ch`. */
	width: number;
}

export function createOverlayCaptions(initial: ReadingSettings) {
	let current = $state<Partial<Record<Origin, Caption>>>({});
	let previous = $state<Partial<Record<Origin, string>>>({});
	let history = $state<Partial<Record<Origin, string>>>({});
	let layout = $state(initial.layout);
	let hideFillers = $state(initial.hideFillers);
	let hold = $state(initial.hold);
	let pace = $state(initial.pace);
	let width = $state(initial.width);
	const timers: Partial<Record<Origin, ReturnType<typeof setTimeout>>> = {};

	// Compact keeps captions subtitle-sized. A turn streams until it completes, which during
	// continuous speech can run for many sentences, so only the most recent slice of it is
	// shown. The budget follows the measure, so the block stays the same number of lines
	// however wide the operator sets it; see `captionBudget`.
	const maxChars = $derived(captionBudget(width));

	/** Add a finished turn to an origin's reading context. Stable reading cleans the turn once,
	 *  here, so its history is exactly the text on screen: the overlay trims it at rendered
	 *  line starts (`trimStable`), and a later filler toggle then applies to new turns rather
	 *  than re-wrapping every line already read. */
	function joinHistory(origin: Origin, turn: string): string {
		if (layout !== 'stable') return appendCaptionHistory(history[origin] ?? '', turn);
		return `${history[origin] ?? ''} ${hideFillers ? cleanSpeech(turn) : turn}`;
	}

	/** Auto-hide an origin's caption so the overlay never sits on a stale line over the
	 *  slides. Always re-armed: even when a turn ends on an interim update and no
	 *  turn-complete ever arrives, the line has to disappear on its own. */
	function scheduleExpiry(c: Caption) {
		clearTimeout(timers[c.origin]);
		timers[c.origin] = setTimeout(
			() => {
				if (layout === 'stable') return;
				delete current[c.origin];
				delete previous[c.origin];
				delete history[c.origin];
			},
			c.final ? hold * 1000 : INTERIM_HOLD_MS
		);
	}

	function rescheduleAll() {
		for (const c of Object.values(current)) if (c) scheduleExpiry(c);
	}

	const presenter = createCaptionPresenter(
		(c) => {
			const cur = current[c.origin];
			// A caption for a new turn of this origin: keep the finished text as the dimmed
			// lead-in to the fresh one. A turn can end without ever being flagged final, so this
			// keys off the turn id changing rather than on `cur.final`.
			if (cur && cur.turnId !== c.turnId && cur.text.trim()) {
				previous[c.origin] = cur.text;
				history[c.origin] = joinHistory(c.origin, cur.text);
			}
			current[c.origin] = c;
			scheduleExpiry(c);
		},
		() => pace
	);

	// The context before each origin's current turn, derived apart from `lines` so it is cleaned
	// when a turn joins it rather than on every caption. Stable reading's history is stored
	// already cleaned — see `joinHistory` — and is shown as it is.
	const contextLeads = $derived(
		Object.fromEntries(
			ORIGIN_ORDER.map((origin) => {
				const context = history[origin] ?? '';
				return [origin, layout === 'fit' && hideFillers ? cleanSpeech(context) : context];
			})
		) as Record<Origin, string>
	);

	const lines = $derived<CaptionLine[]>(
		ORIGIN_ORDER.flatMap((origin) => {
			const caption = current[origin];
			if (!caption) return [];
			const text =
				layout !== 'compact'
					? layout === 'stable'
						? caption.text
						: caption.text.slice(-12000)
					: tail(caption.text, maxChars);
			const room = maxChars - text.length;
			const lead =
				layout !== 'compact'
					? contextLeads[origin]
					: room >= MIN_LEAD_CHARS
						? tail(previous[origin] ?? '', room)
						: '';
			return [
				{
					origin,
					lead: layout === 'compact' && hideFillers ? cleanSpeech(lead) : lead,
					text: hideFillers ? cleanSpeech(text, caption.final) : text,
					interim: !caption.final
				}
			];
		})
	);

	function clearAll() {
		current = {};
		previous = {};
		history = {};
		for (const timer of Object.values(timers)) clearTimeout(timer);
	}

	return {
		get lines() {
			return lines;
		},
		get layout() {
			return layout;
		},
		get width() {
			return width;
		},
		/** A caption event from the core, paced by the reading preference. */
		push(caption: Caption) {
			presenter.push(caption);
		},
		/** A session or source ending: commit or drop what is buffered, and in Stable reading
		 *  clear the retained context when the whole session stops. */
		status(status: StatusUpdate) {
			if (status.state === 'idle' || status.state === 'error') {
				if (!status.origin && layout !== 'stable') presenter.flush();
				presenter.clear(status.origin);
			}
			if (layout === 'stable' && status.state === 'idle' && !status.origin) clearAll();
		},
		setHold(seconds: number) {
			hold = holdSeconds(seconds);
			rescheduleAll();
		},
		setPace(next: CaptionPace) {
			pace = next;
			presenter.flush();
		},
		setHideFillers(hide: boolean) {
			hideFillers = hide;
		},
		setLayout(next: CaptionLayout) {
			if (next === layout) return;
			layout = next;
			if (layout !== 'stable') {
				rescheduleAll();
				for (const origin of ORIGIN_ORDER)
					history[origin] = appendCaptionHistory('', history[origin] ?? '');
			} else if (hideFillers) {
				// Stable reading keeps its history cleaned; context carried over from another
				// layout is cleaned once, on the way in.
				for (const origin of ORIGIN_ORDER)
					if (history[origin]) history[origin] = cleanSpeech(history[origin]);
			}
		},
		setWidth(ch: number) {
			width = clampOverlayWidth(ch);
		},
		/** Drop the first `chars` characters of Stable reading's context, as measured at a
		 *  rendered line start by the caption line. */
		trimStable(origin: Origin, chars: number) {
			history[origin] = (history[origin] ?? '').slice(chars);
		},
		/** Show fixed content, for a browser preview with no core behind it. */
		show(content: {
			current: Partial<Record<Origin, Caption>>;
			previous?: Partial<Record<Origin, string>>;
			history?: Partial<Record<Origin, string>>;
		}) {
			current = content.current;
			previous = content.previous ?? {};
			history = content.history ?? {};
		},
		dispose() {
			presenter.clear();
			for (const timer of Object.values(timers)) clearTimeout(timer);
		}
	};
}
export type OverlayCaptions = ReturnType<typeof createOverlayCaptions>;
