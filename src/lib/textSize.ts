// The operator window's own text size (issue #283): Settings › App › Text size.
//
// A percentage the operator picks, multiplied with the system's text size by `textScale.ts`.
// It is relative on purpose: on Windows, *Make text bigger* still counts, and 100% is exactly
// what the window did before this setting existed, so the default needs no separate "System"
// choice that would draw the same window. On macOS, which reports no text size to the webview,
// this is the only factor there is.
//
// Kept apart from `textScale.ts`, which talks to the core, so the settings panel can read the
// choices without pulling in the listener — and without tripping over tests that mock it.

import { persisted, readStored } from './persisted';

/** The choices, in percent. 90 is the one step below the system's size, checked against the
 *  layout in docs/accessibility.md; the steps above stop where 150% of Windows' 150% already
 *  reaches the 225% the layout is verified at. */
export const TEXT_SIZES = [90, 100, 115, 130, 150] as const;
export type TextSize = (typeof TEXT_SIZES)[number];
export const TEXT_SIZE_DEFAULT: TextSize = 100;

export const TEXT_SIZE_KEY = 'appearance.textSize';

/** The stored choice, or 100 for nothing or anything that is not one of the choices. */
export function loadTextSize(): TextSize {
	const stored = Number(readStored(TEXT_SIZE_KEY));
	return TEXT_SIZES.find((size) => size === stored) ?? TEXT_SIZE_DEFAULT;
}

export const textSize = persisted<TextSize>(TEXT_SIZE_KEY, loadTextSize);
