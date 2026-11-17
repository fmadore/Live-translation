// The opt-in system-wide keys (`src-tauri/src/hotkeys.rs`): what the Settings row prints and
// announces. The core registers and hears them; this window only says which they are and
// whether the system let the app have them.

import { writable } from 'svelte/store';
import { isMac } from './platform';
import type { GlobalShortcut, GlobalShortcutStatus } from './types';

/** A key of a system-wide combination, as the catalog names it (`$t.keys`). `control` is the
 *  Control key on every system, unlike the window shortcuts' `ctrl`, which is Command on a
 *  Mac: the system-wide keys are ⌃⌥⇧ there, a set Mac apps leave alone. */
export type GlobalKeyName = 'control' | 'alt' | 'shift' | 'p' | 'o';

/** Each combination: the window's own P and O with Alt added. Ctrl+Alt+Shift is a set that
 *  PowerPoint, Keynote, Zoom, Teams and the browsers do not use, where Ctrl+Shift+P opens a
 *  private browser window. `contract.test.ts` holds this to the core's accelerators. */
export const GLOBAL_SHORTCUT_KEYS: Record<GlobalShortcut, readonly GlobalKeyName[]> = {
	'toggle-pause': ['control', 'alt', 'shift', 'p'],
	'toggle-overlay': ['control', 'alt', 'shift', 'o']
};

/** ARIA's names for the keys, whatever the interface language. */
const ARIA: Record<GlobalKeyName, string> = {
	control: 'Control',
	alt: 'Alt',
	shift: 'Shift',
	p: 'P',
	o: 'O'
};

/** The keys in the order the system writes them: Ctrl, Alt, Shift on Windows; Control, Option,
 *  Shift (⌃⌥⇧) on a Mac, which is also Apple's order. The table already holds both. */
export function globalKeyParts(
	shortcut: GlobalShortcut,
	names: Record<GlobalKeyName, string>
): string[] {
	return GLOBAL_SHORTCUT_KEYS[shortcut].map((key) => names[key]);
}

/** The combination as the operator reads it: `Ctrl+Alt+Shift+P`, or `⌃⌥⇧P` on a Mac. */
export function globalKeyLabel(
	shortcut: GlobalShortcut,
	names: Record<GlobalKeyName, string>,
	mac = isMac
): string {
	return globalKeyParts(shortcut, names).join(mac ? '' : '+');
}

/** The value for `aria-keyshortcuts`. */
export function ariaGlobalShortcut(shortcut: GlobalShortcut): string {
	return GLOBAL_SHORTCUT_KEYS[shortcut].map((key) => ARIA[key]).join('+');
}

/** What the core last answered while the keys were wanted (`nativeSync`): kept after Stop, so
 *  a refusal can still be read in Settings, and emptied when the switch goes off. */
export const globalShortcutStatus = writable<GlobalShortcutStatus>({ registered: [], failed: [] });

/** The refused combinations as one phrase, joined with the language's “or”. */
export function refusedPhrase(labels: readonly string[], localeTag: string): string {
	return new Intl.ListFormat(localeTag, { type: 'disjunction' }).format(labels);
}

/** The combinations the system refused, as the operator reads them, in table order. */
export function refusedLabels(
	status: GlobalShortcutStatus,
	names: Record<GlobalKeyName, string>,
	mac = isMac
): string[] {
	return (Object.keys(GLOBAL_SHORTCUT_KEYS) as GlobalShortcut[])
		.filter((shortcut) => status.failed.includes(shortcut))
		.map((shortcut) => globalKeyLabel(shortcut, names, mac));
}
