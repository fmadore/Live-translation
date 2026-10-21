import { afterEach, describe, expect, it } from 'vitest';
import { applyTheme, loadTheme, resolveTheme } from './theme';

function useStorage(values: Record<string, string>) {
	Object.defineProperty(globalThis, 'localStorage', {
		value: { getItem: (key: string) => values[key] ?? null, setItem: () => {} },
		configurable: true
	});
}

afterEach(() => {
	Reflect.deleteProperty(globalThis, 'localStorage');
});

describe('the theme', () => {
	it('follows Windows until the operator picks one', () => {
		expect(loadTheme()).toBe('system');
		useStorage({ 'appearance.theme': 'light' });
		expect(loadTheme()).toBe('light');
	});

	// Storage is shared with the overlay and outlives updates, so it can hold anything.
	it('reads anything it does not know as "system"', () => {
		useStorage({ 'appearance.theme': 'sepia' });
		expect(loadTheme()).toBe('system');
	});

	it('resolves "system" from the app mode and leaves an explicit choice alone', () => {
		expect(resolveTheme('system', true)).toBe('light');
		expect(resolveTheme('system', false)).toBe('dark');
		expect(resolveTheme('dark', true)).toBe('dark');
		expect(resolveTheme('light', false)).toBe('light');
	});

	// Always an explicit value: app.css has no `prefers-color-scheme` copy of the light block,
	// so "system" has to arrive at the root already resolved.
	it('writes the resolved theme onto the root', () => {
		const attributes: Record<string, string> = {};
		const root = { setAttribute: (name: string, value: string) => (attributes[name] = value) };
		applyTheme('light', root);
		expect(attributes).toEqual({ 'data-theme': 'light' });
		applyTheme('dark', root);
		expect(attributes).toEqual({ 'data-theme': 'dark' });
	});
});
