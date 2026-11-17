// A notice on the overlay for a break: one short line the room reads instead of the captions
// ("Back soon"), until the operator clears it or, by default, until captions come back after a
// pause. It travels in the overlay's configuration push like the rest of what the operator
// sets (`OverlayConfig.notice`), so the overlay needs no new permission, and it is plain text:
// the overlay renders it as text and checks it again as it arrives. It is never written to the
// transcript or sent to Zoom.

import { isLocale, type Locale } from './i18n';
import { plainLine } from './plainText';
import type { SessionState, TargetLanguage } from './types';

/** The longest notice, in characters: one line at a reading size, not a slide. */
export const NOTICE_MAX_CHARS = 80;

/** A notice as it is shown: one line of plain text, at most `NOTICE_MAX_CHARS` long. Anything
 *  else, including nothing, is no notice. Applied on both sides of the push. */
export function normalizeNotice(value: unknown): string {
	return plainLine(value, NOTICE_MAX_CHARS);
}

/** The suggestions offered beside the text field, by their catalog key. */
export const NOTICE_PRESETS = ['paused', 'backSoon', 'shortBreak'] as const;

/** The language the suggestions are offered in: they are for the room, so the caption
 *  language when the app has a catalog for it, and the interface language otherwise. */
export function noticeLocale(
	captionLanguage: TargetLanguage | undefined,
	fallback: Locale
): Locale {
	const primary = captionLanguage?.split('-')[0];
	return isLocale(primary) ? primary : fallback;
}

/** Whether a notice goes by itself at the first caption after a pause (`noticeUntilCaptions`
 *  in `preferences.ts`). */
export const NOTICE_UNTIL_CAPTIONS_KEY = 'overlay.noticeUntilCaptions';

/**
 * The notice and whether the next caption clears it.
 *
 * A notice put up for a break should not outlast it, but one shown while people are still
 * talking — over a video clip, say — should not vanish with the next word either. So a caption
 * clears it only once captions have stopped since it went up: the session was paused, or was
 * not running at all. Then the first caption is the break being over.
 */
export interface NoticeState {
	text: string;
	/** Captions have stopped since the notice went up: the next one clears it, when asked to. */
	armed: boolean;
}

export const NO_NOTICE: NoticeState = Object.freeze({ text: '', armed: false });

/** Whether a session in `state` is not sending captions: paused, or not running. */
function quiet(state: SessionState): boolean {
	return state === 'paused' || state === 'idle' || state === 'error';
}

/** Put up `text` while the session is in `state`. Empty text takes the notice down. */
export function showNotice(text: string, state: SessionState): NoticeState {
	const notice = normalizeNotice(text);
	return notice ? { text: notice, armed: quiet(state) } : NO_NOTICE;
}

/** The session moved to `state`. */
export function noticeOnState(notice: NoticeState, state: SessionState): NoticeState {
	return notice.text && !notice.armed && quiet(state) ? { ...notice, armed: true } : notice;
}

/** A caption arrived. `untilCaptions` is the operator's choice to let captions clear it. */
export function noticeOnCaption(notice: NoticeState, untilCaptions: boolean): NoticeState {
	return notice.text && notice.armed && untilCaptions ? NO_NOTICE : notice;
}
