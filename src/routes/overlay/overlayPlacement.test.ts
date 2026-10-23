import { describe, expect, it } from 'vitest';
import { overlayKeyCommand } from './overlayPlacement.svelte';

const key = (name: string, extra: { shiftKey?: boolean; onButton?: boolean } = {}) => ({
	key: name,
	shiftKey: extra.shiftKey ?? false,
	onButton: extra.onButton ?? false
});

describe('overlay keys', () => {
	it('change the caption size in either mode', () => {
		for (const interactive of [false, true]) {
			expect(overlayKeyCommand(key('+'), interactive)).toEqual({ kind: 'bump', delta: 2 });
			expect(overlayKeyCommand(key('='), interactive)).toEqual({ kind: 'bump', delta: 2 });
			expect(overlayKeyCommand(key('-'), interactive)).toEqual({ kind: 'bump', delta: -2 });
		}
	});

	it('do nothing else outside move mode', () => {
		for (const name of ['Enter', 'Escape', 'ArrowLeft', 'a'])
			expect(overlayKeyCommand(key(name), false)).toBeNull();
	});

	it('offer both ways out of move mode', () => {
		expect(overlayKeyCommand(key('Enter'), true)).toEqual({ kind: 'lock' });
		expect(overlayKeyCommand(key('Escape'), true)).toEqual({ kind: 'cancel' });
	});

	// A focused toolbar button already answers Enter; the shortcut must not run it twice.
	it('leave Enter to a focused button', () => {
		expect(overlayKeyCommand(key('Enter', { onButton: true }), true)).toBeNull();
		expect(overlayKeyCommand(key('Escape', { onButton: true }), true)).toEqual({
			kind: 'cancel'
		});
	});

	it('nudge by one pixel, or ten with Shift', () => {
		expect(overlayKeyCommand(key('ArrowLeft'), true)).toEqual({ kind: 'nudge', dx: -1, dy: 0 });
		expect(overlayKeyCommand(key('ArrowDown', { shiftKey: true }), true)).toEqual({
			kind: 'nudge',
			dx: 0,
			dy: 10
		});
		expect(overlayKeyCommand(key('ArrowUp'), true)).toEqual({ kind: 'nudge', dx: 0, dy: -1 });
		expect(overlayKeyCommand(key('ArrowRight', { shiftKey: true }), true)).toEqual({
			kind: 'nudge',
			dx: 10,
			dy: 0
		});
	});
});
