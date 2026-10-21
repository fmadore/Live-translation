// The operator window's colour theme: Windows' app mode, or a choice that overrides it.
//
// app.css draws the dark palette by default and re-declares its colours under
// `:root[data-theme='light']`, so all this module does is decide which of the two applies and
// write it onto the document root — always as an explicit `light` or `dark`, so the stylesheet
// never has to repeat the light block inside a `prefers-color-scheme` query to cover "follow
// the system".
//
// The overlay does not call this. Its captions have their own palette, chosen for the room,
// and its move-mode chrome sits over a slide nobody controls; with no attribute on its root it
// keeps the dark tokens whatever the operator picks, the same reason it opts out of the text
// size and of contrast themes.

import { persisted, readStored } from './persisted';
import { isTauri } from './tauri';

export const THEMES = ['system', 'light', 'dark'] as const;
export type ThemeChoice = (typeof THEMES)[number];
export type Theme = Exclude<ThemeChoice, 'system'>;

const THEME_KEY = 'appearance.theme';

/** The stored choice, or `system` for nothing or anything this version does not know. */
export function loadTheme(): ThemeChoice {
	const stored = readStored(THEME_KEY);
	return THEMES.find((choice) => choice === stored) ?? 'system';
}

export const theme = persisted<ThemeChoice>(THEME_KEY, loadTheme);

/** What a choice paints, given whether Windows is in light app mode. */
export function resolveTheme(choice: ThemeChoice, systemLight: boolean): Theme {
	if (choice === 'system') return systemLight ? 'light' : 'dark';
	return choice;
}

/** Just enough of an element to carry the attribute — `HTMLElement` satisfies it, and so does a
 *  stand-in, which keeps this module's tests out of a DOM environment. */
export type ThemeTarget = { setAttribute(name: string, value: string): void };

export function applyTheme(resolved: Theme, root?: ThemeTarget): void {
	const target = root ?? (typeof document === 'undefined' ? undefined : document.documentElement);
	target?.setAttribute('data-theme', resolved);
}

/**
 * Apply the operator's theme and keep following both the setting and Windows. Returns an
 * unsubscribe.
 *
 * Synchronous on purpose, and called at component init rather than in `onMount`: the attribute
 * has to be on the root before the first paint, or someone on a light desktop would be shown a
 * frame of the dark window first.
 *
 * The native title bar is told too. `null` hands it back to Windows, so on "system" it follows
 * the app mode by itself instead of through this listener.
 */
export function followTheme(): () => void {
	const query =
		typeof matchMedia === 'undefined' ? undefined : matchMedia('(prefers-color-scheme: light)');
	let choice: ThemeChoice = 'system';
	const apply = () => applyTheme(resolveTheme(choice, query?.matches ?? false));
	const onSystem = () => {
		if (choice === 'system') apply();
	};

	query?.addEventListener('change', onSystem);
	const stop = theme.subscribe((next) => {
		choice = next;
		apply();
		void titleBar(next);
	});
	return () => {
		stop();
		query?.removeEventListener('change', onSystem);
	};
}

async function titleBar(choice: ThemeChoice): Promise<void> {
	if (!isTauri()) return;
	try {
		const { getCurrentWindow } = await import('@tauri-apps/api/window');
		await getCurrentWindow().setTheme(choice === 'system' ? null : choice);
	} catch {
		// A title bar left in the other mode is cosmetic; the window itself has already changed.
	}
}
