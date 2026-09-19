// @vitest-environment jsdom
import { expect, it } from 'vitest';
import { shortcut } from './shortcuts';
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
