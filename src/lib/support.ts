// Voluntary support (issue #180). Everything the app does stays free; this is a link, shown
// in two quiet places — the App tab of Settings and the idle stage's credit line — and never
// while a session runs.

import { readable } from 'svelte/store';
import { isMac } from './platform';
import { api, isTauri } from './tauri';

/** The creator's Buy Me a Coffee page. The core opens this exact page and no other
 *  (`src-tauri/src/support.rs`, whose test keeps the two in step). */
export const SUPPORT_URL = 'https://buymeacoffee.com/fmadore';

/** Open the support page in the default browser. In the desktop app the core does it, since
 *  WebView2 would open a `target="_blank"` link in a popup of its own; a browser preview has
 *  no core and opens a tab instead. */
export async function openSupportPage(): Promise<void> {
	if (isTauri()) {
		await api.openSupportPage();
	} else {
		window.open(SUPPORT_URL, '_blank', 'noopener,noreferrer');
	}
}

/** Whether to show the link. Always on Windows and in a browser preview; on a Mac the core
 *  says, since the Mac App Store build does not offer it (App Review Guideline 3.1.1). Hidden
 *  on a Mac until it has answered, so the store build never shows it for a moment. */
export const supportShown = readable(!isMac || !isTauri(), (set) => {
	if (!isMac || !isTauri()) return;
	api.supportLinkAllowed().then(set, () => set(false));
});
