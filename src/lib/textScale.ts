// The operator window's text size: the system's, times the operator's own choice.
//
// Windows' *Make text bigger* slider does not reach web content inside WebView2, so the Rust
// side reads `UISettings.TextScaleFactor` and hands it over — see `src-tauri/src/textscale.rs`
// and docs/accessibility.md. macOS reports nothing, so the core answers 1 there. Settings ›
// App › Text size (`textSize.ts`) multiplies whichever it is, and this module writes the
// product onto one custom property; the `--type-*` ramp in app.css multiplies every font size
// in the window by it, and `:root`'s own font size carries it into the `em` breakpoints that
// decide when the two columns stack.
//
// The overlay does not call this. Its captions are projected content sized for a room by the
// operator, the same reason it opts out of contrast themes.

import { api, isTauri, on } from './tauri';
import { textSize, TEXT_SIZES } from './textSize';

/** The range of the Windows slider; anything outside it is a corrupt read, not a request. */
const SYSTEM_SCALE_MIN = 1;
/** The largest factor the layout is verified at, whatever combination asks for more. */
export const TEXT_SCALE_MAX = 2.25;
/** The smallest: the lowest Text size on a system at 100%. */
export const TEXT_SCALE_MIN = Math.min(...TEXT_SIZES) / 100;
export const TEXT_SCALE_DEFAULT = 1;

/** A factor held to the range, or no scaling for anything that is not a real number. */
function clamp(factor: unknown, min: number): number {
	if (typeof factor !== 'number' || !Number.isFinite(factor)) return TEXT_SCALE_DEFAULT;
	return Math.min(TEXT_SCALE_MAX, Math.max(min, factor));
}

/**
 * Hold the system's factor to the range of the Windows slider.
 *
 * The core clamps too. This is not redundant: the value also arrives from an event payload
 * that has crossed a JSON boundary, and a `NaN` here would invalidate every `calc()` in the
 * stylesheet at once and leave the operator looking at an unstyled window.
 */
export function clampTextScale(factor: unknown): number {
	return clamp(factor, SYSTEM_SCALE_MIN);
}

/**
 * The factor the window is drawn at: the system's, times the chosen Text size in percent,
 * held to the range the layout is verified at. Each input is checked on its own first, so a
 * corrupt system read still honours the operator's choice and a corrupt choice still honours
 * the system. 150% of Windows' 150% is the 225% ceiling; 150% of 200% is held there too.
 */
export function combinedTextScale(system: unknown, percent: unknown): number {
	const choice = clamp(typeof percent === 'number' ? percent / 100 : NaN, TEXT_SCALE_MIN);
	// Rounded so the stylesheet reads 1.725, not 1.7249999999999999.
	return clamp(Math.round(clampTextScale(system) * choice * 1000) / 1000, TEXT_SCALE_MIN);
}

/** Just enough of an element to carry a custom property — `HTMLElement` satisfies it, and so
 *  does a stand-in, which is what keeps this module's tests out of a DOM environment. */
export type StyleTarget = { style: { setProperty(name: string, value: string): void } };

/** Write the factor onto the document root, where app.css picks it up. */
export function applyTextScale(factor: unknown, root?: StyleTarget): void {
	const target = root ?? (typeof document === 'undefined' ? undefined : document.documentElement);
	if (!target) return;
	target.style.setProperty('--text-scale', String(clamp(factor, TEXT_SCALE_MIN)));
}

/**
 * Apply the operator's text size and keep following it, the system's and the chosen one
 * alike. Returns an unsubscribe.
 *
 * The chosen size is applied at once, from storage. The system's is asked for rather than
 * waited for: the window has to lay itself out before it could have subscribed to anything,
 * and someone who needs 225% text should not be shown a frame of 9.5px type first. In a
 * browser preview there is no system to ask, and the chosen size alone applies.
 */
export async function followTextScale(): Promise<() => void> {
	let system: unknown = TEXT_SCALE_DEFAULT;
	let percent: unknown;
	const apply = () => applyTextScale(combinedTextScale(system, percent));
	const stopSize = textSize.subscribe((value) => {
		percent = value;
		apply();
	});
	if (!isTauri()) return stopSize;

	const stopSystem = await on.textScale((factor) => {
		system = factor;
		apply();
	});
	try {
		system = await api.textScaleFactor();
		apply();
	} catch {
		// A failed read is not worth an error in front of the operator: the window is usable
		// at the chosen size alone, and the slider still reports any later change through the
		// event.
	}
	return () => {
		stopSize();
		stopSystem();
	};
}
