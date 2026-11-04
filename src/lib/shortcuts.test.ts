// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';
import {
	ariaKeyShortcut,
	keyLabel,
	keyParts,
	shortcut,
	SHORTCUT_KEYS,
	type KeyName,
	type Shortcut
} from './shortcuts';
import { en } from './i18n/en';
import { de } from './i18n/de';
import { fr } from './i18n/fr';
import { catalogsFor } from './i18n';
it('requires the modifier combination and suppresses repeat, dialogs and text editing', () => {
	const event = new KeyboardEvent('keydown', { code: 'Space', ctrlKey: true, shiftKey: true });
	expect(shortcut(event, false)).toBe('toggleSession');
	expect(shortcut(event, true)).toBeNull();
	expect(shortcut(new KeyboardEvent('keydown', { code: 'Space' }), false)).toBeNull();
	expect(
		shortcut(
			new KeyboardEvent('keydown', { code: 'KeyO', ctrlKey: true, shiftKey: true, repeat: true }),
			false
		)
	).toBeNull();
	const input = document.createElement('input');
	Object.defineProperty(event, 'target', { value: input });
	expect(shortcut(event, false)).toBeNull();
});

// The table is the one place a shortcut is written down; the listener, the ARIA attribute and
// the printed key caps all read it. These pin the three to each other.
describe('the shortcut table', () => {
	const EVENT: Record<Exclude<KeyName, 'ctrl' | 'shift'>, KeyboardEventInit> = {
		space: { code: 'Space' },
		p: { code: 'KeyP' },
		o: { code: 'KeyO' },
		up: { code: 'ArrowUp' },
		down: { code: 'ArrowDown' },
		f2: { key: 'F2' }
	};

	it.each(Object.keys(SHORTCUT_KEYS) as Shortcut[])(
		'is what the listener answers: %s',
		(command) => {
			const keys = SHORTCUT_KEYS[command];
			const main = keys.find((key) => key !== 'ctrl' && key !== 'shift') as keyof typeof EVENT;
			const init = {
				...EVENT[main],
				ctrlKey: keys.includes('ctrl'),
				shiftKey: keys.includes('shift')
			};
			expect(shortcut(new KeyboardEvent('keydown', init), false)).toBe(command);
			// One modifier short is not the shortcut.
			if (keys.includes('shift')) {
				expect(
					shortcut(new KeyboardEvent('keydown', { ...init, shiftKey: false }), false)
				).toBeNull();
			}
		}
	);

	it('announces in ARIA key names, whatever the interface language', () => {
		expect(ariaKeyShortcut('toggleSession')).toBe('Control+Shift+Space');
		expect(ariaKeyShortcut('toggleOverlay')).toBe('Control+Shift+O');
		expect(ariaKeyShortcut('larger')).toBe('Control+Shift+ArrowUp');
		expect(ariaKeyShortcut('direction')).toBe('F2');
	});

	it('prints in the key names of the interface language, the way Windows writes them', () => {
		expect(keyLabel('toggleSession', en.keys)).toBe('Ctrl+Shift+Space');
		expect(keyLabel('toggleSession', fr.keys)).toBe('Ctrl+Maj+Espace');
		expect(keyLabel('toggleSession', de.keys)).toBe('Strg+Umschalt+Leertaste');
		expect(keyLabel('smaller', en.keys)).toBe('Ctrl+Shift+↓');
	});
});

// On a Mac the table's `ctrl` is Command: the listener answers ⌘, never Control, and the keys
// are written as macOS writes them, in the catalog's Mac names.
describe('on a Mac', () => {
	const mac = catalogsFor(true);

	it('answers Command, and leaves Control alone', () => {
		const command = new KeyboardEvent('keydown', { code: 'Space', metaKey: true, shiftKey: true });
		expect(shortcut(command, false, true)).toBe('toggleSession');
		const control = new KeyboardEvent('keydown', { code: 'Space', ctrlKey: true, shiftKey: true });
		expect(shortcut(control, false, true)).toBeNull();
		// And Windows the other way round: the Windows key is never the modifier.
		expect(shortcut(command, false, false)).toBeNull();
		expect(shortcut(new KeyboardEvent('keydown', { key: 'F2' }), false, true)).toBe('direction');
	});

	it('announces Command as Meta', () => {
		expect(ariaKeyShortcut('togglePause', true)).toBe('Meta+Shift+P');
		expect(ariaKeyShortcut('direction', true)).toBe('F2');
	});

	it('writes Shift before Command, with nothing between the keys', () => {
		expect(keyLabel('togglePause', mac.en.keys, true)).toBe('⇧⌘P');
		expect(keyLabel('toggleSession', mac.fr.keys, true)).toBe('⇧⌘Espace');
		expect(keyLabel('larger', mac.de.keys, true)).toBe('⇧⌘↑');
		expect(keyParts('toggleOverlay', mac.en.keys, true)).toEqual(['⇧', '⌘', 'O']);
		expect(keyLabel('direction', mac.en.keys, true)).toBe('F2');
	});
});
