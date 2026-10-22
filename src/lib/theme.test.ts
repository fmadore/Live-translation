import { readFileSync } from 'node:fs';
import { afterEach, describe, expect, it } from 'vitest';
import { applyTheme, followTheme, loadTheme, resolveTheme, theme, THEME_KEY } from './theme';

function useStorage(values: Record<string, string>) {
	Object.defineProperty(globalThis, 'localStorage', {
		value: { getItem: (key: string) => values[key] ?? null, setItem: () => {} },
		configurable: true
	});
}

/** Windows' app mode as `matchMedia` reports it, switchable mid-test. */
function useAppMode(light: boolean) {
	const listeners = new Set<() => void>();
	const query = {
		matches: light,
		addEventListener: (_: 'change', listener: () => void) => listeners.add(listener),
		removeEventListener: (_: 'change', listener: () => void) => listeners.delete(listener)
	};
	Object.defineProperty(globalThis, 'matchMedia', {
		value: (media: string) => {
			expect(media).toBe('(prefers-color-scheme: light)');
			return query;
		},
		configurable: true
	});
	return {
		listeners,
		switchTo(next: boolean) {
			query.matches = next;
			for (const listener of listeners) listener();
		}
	};
}

/** A document whose root only records `data-theme`, for code that reaches for the real one. */
function useDocument() {
	const root = { theme: '', setAttribute: (_: string, value: string) => (root.theme = value) };
	Object.defineProperty(globalThis, 'document', {
		value: { documentElement: root },
		configurable: true
	});
	return root;
}

afterEach(() => {
	theme.set('system');
	for (const name of ['localStorage', 'matchMedia', 'document', 'location']) {
		Reflect.deleteProperty(globalThis, name);
	}
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

describe('following the theme', () => {
	it('tracks Windows on "system" and stops when unsubscribed', () => {
		const mode = useAppMode(false);
		const root = useDocument();
		const stop = followTheme();
		expect(root.theme).toBe('dark');

		mode.switchTo(true);
		expect(root.theme).toBe('light');

		stop();
		expect(mode.listeners.size).toBe(0);
		mode.switchTo(false);
		expect(root.theme).toBe('light');
	});

	it('holds an explicit choice whatever Windows does, and lets go on "system"', () => {
		const mode = useAppMode(true);
		const root = useDocument();
		const stop = followTheme();

		theme.set('dark');
		expect(root.theme).toBe('dark');
		mode.switchTo(false);
		mode.switchTo(true);
		expect(root.theme).toBe('dark');

		theme.set('system');
		expect(root.theme).toBe('light');
		stop();
	});
});

// `src/app.html` runs a copy of `loadTheme` + `resolveTheme` before anything loads, so the
// first frame is right. This runs that copy against stand-ins, so the two cannot drift apart.
describe('the first-frame script in app.html', () => {
	const html = readFileSync(new URL('../app.html', import.meta.url), 'utf8');
	const script = html.match(/<script>([\s\S]*?)<\/script>/)![1];

	function firstFrame(path: string, stored: string | null, systemLight: boolean) {
		useStorage(stored === null ? {} : { [THEME_KEY]: stored });
		useAppMode(systemLight);
		const root = useDocument();
		Object.defineProperty(globalThis, 'location', {
			value: { pathname: path },
			configurable: true
		});
		new Function(script)();
		return root.theme;
	}

	it('reads the setting under the key theme.ts stores it under', () => {
		expect(script).toContain(`localStorage.getItem('${THEME_KEY}')`);
	});

	it('resolves the setting the way theme.ts does', () => {
		for (const stored of [null, 'system', 'light', 'dark', 'sepia']) {
			for (const systemLight of [true, false]) {
				useStorage(stored === null ? {} : { [THEME_KEY]: stored });
				const expected = resolveTheme(loadTheme(), systemLight);
				expect(firstFrame('/', stored, systemLight), `${stored}, light=${systemLight}`).toBe(
					expected
				);
			}
		}
	});

	it('leaves the overlay alone', () => {
		expect(firstFrame('/overlay/', 'light', true)).toBe('');
	});

	it('still paints when storage refuses', () => {
		useAppMode(true);
		const root = useDocument();
		Object.defineProperty(globalThis, 'localStorage', {
			get() {
				throw new Error('blocked');
			},
			configurable: true
		});
		Object.defineProperty(globalThis, 'location', { value: { pathname: '/' }, configurable: true });
		new Function(script)();
		expect(root.theme).toBe('light');
	});
});
