import { describe, expect, it } from 'vitest';
import {
	ariaGlobalShortcut,
	GLOBAL_SHORTCUT_KEYS,
	globalKeyLabel,
	refusedLabels,
	refusedPhrase
} from './globalShortcuts';
import { SHORTCUT_KEYS } from './shortcuts';
import { de, en, fr } from './i18n';
import { macEn } from './i18n/mac';
import { GLOBAL_SHORTCUTS } from './types';

const macKeys = { ...en.keys, ...macEn.keys };

describe('system-wide shortcuts', () => {
	it('are Ctrl+Alt+Shift on Windows and ⌃⌥⇧ on a Mac', () => {
		expect(globalKeyLabel('toggle-pause', en.keys, false)).toBe('Ctrl+Alt+Shift+P');
		expect(globalKeyLabel('toggle-overlay', de.keys, false)).toBe('Strg+Alt+Umschalt+O');
		expect(globalKeyLabel('toggle-overlay', fr.keys, false)).toBe('Ctrl+Alt+Maj+O');
		expect(globalKeyLabel('toggle-pause', macKeys, true)).toBe('⌃⌥⇧P');
		expect(ariaGlobalShortcut('toggle-pause')).toBe('Control+Alt+Shift+P');
	});

	/** The window's own Pause and Show/hide overlay keys, with one more modifier: easy to
	 *  remember, and never the same combination as a window shortcut. */
	it('extend the window shortcuts of the same name and never repeat one', () => {
		expect(GLOBAL_SHORTCUT_KEYS['toggle-pause'].at(-1)).toBe(SHORTCUT_KEYS.togglePause.at(-1));
		expect(GLOBAL_SHORTCUT_KEYS['toggle-overlay'].at(-1)).toBe(SHORTCUT_KEYS.toggleOverlay.at(-1));
		for (const shortcut of GLOBAL_SHORTCUTS) {
			expect(GLOBAL_SHORTCUT_KEYS[shortcut]).toContain('alt');
			expect(GLOBAL_SHORTCUT_KEYS[shortcut]).toContain('control');
		}
	});

	it('name the refused keys in table order, and nothing when none was refused', () => {
		expect(refusedLabels({ registered: ['toggle-pause'], failed: [] }, en.keys, false)).toEqual([]);
		expect(
			refusedLabels({ registered: [], failed: ['toggle-overlay', 'toggle-pause'] }, en.keys, false)
		).toEqual(['Ctrl+Alt+Shift+P', 'Ctrl+Alt+Shift+O']);
	});

	it('word a refusal in each language', () => {
		const two = ['Ctrl+Alt+Shift+P', 'Ctrl+Alt+Shift+O'];
		expect(refusedPhrase(two, 'en')).toBe('Ctrl+Alt+Shift+P or Ctrl+Alt+Shift+O');
		expect(refusedPhrase(two, 'fr')).toBe('Ctrl+Alt+Shift+P ou Ctrl+Alt+Shift+O');
		expect(refusedPhrase(['Strg+Alt+Umschalt+O'], 'de')).toBe('Strg+Alt+Umschalt+O');
		expect(en.usability.globalRefused('Ctrl+Alt+Shift+O', 1)).toMatch(
			/^Could not register Ctrl\+Alt\+Shift\+O: .* uses it\./
		);
		expect(fr.usability.globalRefused('A ou B', 2)).toContain('les utilise');
		expect(de.usability.globalRefused('A', 1)).toContain('konnte nicht');
		expect(de.usability.globalRefused('A oder B', 2)).toContain('konnten nicht');
	});
});
