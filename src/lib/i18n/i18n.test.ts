import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { en } from './en';
import { fr } from './fr';
import { get } from 'svelte/store';
import { detectLocale, locale, t, localeTag, LOCALE_KEY, LOCALES, formatDateTime } from './index';

/** Every leaf in a catalog, as `a.b.c` paths, so two catalogs can be compared as sets. */
function paths(value: unknown, prefix = ''): string[] {
	if (typeof value !== 'object' || value === null) return [prefix];
	return Object.entries(value).flatMap(([key, child]) =>
		paths(child, prefix ? `${prefix}.${key}` : key)
	);
}

/** The leaf at `a.b.c`. */
function at(catalog: unknown, path: string): unknown {
	return path.split('.').reduce<unknown>((node, key) => (node as never)[key], catalog);
}

describe('the message catalogs', () => {
	// TypeScript already fails a catalog with a missing or extra key — `fr` is typed as
	// `typeof en`. This is the pass structural typing cannot make: a key that is present and
	// typed correctly but empty, or a message that is a string where English takes arguments.
	it('agree on every key', () => {
		expect(paths(fr).sort()).toEqual(paths(en).sort());
	});

	it('agree on which messages take parameters', () => {
		for (const path of paths(en)) {
			const source = at(en, path);
			const target = at(fr, path);
			expect(
				typeof target,
				`${path} is a ${typeof target} in fr and a ${typeof source} in en`
			).toBe(typeof source);
			if (typeof source === 'function' && typeof target === 'function') {
				expect(target.length, `${path} takes a different number of arguments in fr`).toBe(
					source.length
				);
			}
		}
	});

	it('leave nothing blank', () => {
		for (const catalog of [en, fr]) {
			for (const path of paths(catalog)) {
				const message = at(catalog, path);
				if (typeof message === 'string') {
					expect(message.trim(), `${path} is empty`).not.toBe('');
				}
			}
		}
	});

	it('name each language in its own language', () => {
		expect(en.locale.name).toBe('English');
		expect(fr.locale.name).toBe('Français');
	});

	it('carry a formatting tag per locale', () => {
		expect(en.locale.tag).toBe('en-GB');
		expect(fr.locale.tag).toBe('fr-FR');
	});
});

// French punctuation takes a non-breaking space before `: ? ; !`, and this UI is dense enough
// that the alternative — a line breaking between a word and its question mark — is not
// hypothetical. Invisible in a diff, so checked here.
describe('French typography', () => {
	const strings = (value: unknown): string[] =>
		typeof value === 'string'
			? [value]
			: typeof value === 'object' && value !== null
				? Object.values(value).flatMap(strings)
				: [];

	it('uses a non-breaking space before its double punctuation', () => {
		for (const message of strings(fr)) {
			expect(message, `"${message}" has an ordinary space before its punctuation`).not.toMatch(
				/ [:;?!]/
			);
		}
	});

	it('uses the typographic apostrophe', () => {
		for (const message of strings(fr)) {
			expect(message, `"${message}" uses a straight apostrophe`).not.toMatch(/\w'\w/);
		}
	});
});

describe('choosing a locale', () => {
	const storage = new Map<string, string>();

	// These run in the `logic` project, which has no DOM, so the two globals `detectLocale`
	// reads are stubbed rather than mocked away.
	beforeEach(() => {
		storage.clear();
		vi.stubGlobal('localStorage', {
			getItem: (key: string) => storage.get(key) ?? null,
			setItem: (key: string, value: string) => void storage.set(key, value)
		});
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('follows an explicit choice over the machine', () => {
		storage.set(LOCALE_KEY, 'en');
		vi.stubGlobal('navigator', { languages: ['fr-FR'], language: 'fr-FR' });
		expect(detectLocale()).toBe('en');
	});

	// An operator whose Windows is in French should be spoken to in French on first run,
	// without having to find a selector to say so.
	it('follows the machine on first run', () => {
		vi.stubGlobal('navigator', { languages: ['fr-CA', 'en-US'], language: 'fr-CA' });
		expect(detectLocale()).toBe('fr');
	});

	it('falls back to English for a language it does not have', () => {
		vi.stubGlobal('navigator', { languages: ['de-DE'], language: 'de-DE' });
		expect(detectLocale()).toBe('en');
	});

	it('ignores a stored value that is not a locale it ships', () => {
		storage.set(LOCALE_KEY, 'de');
		vi.stubGlobal('navigator', { languages: ['en-GB'], language: 'en-GB' });
		expect(detectLocale()).toBe('en');
	});

	it('offers exactly the catalogs it has', () => {
		expect(LOCALES).toEqual(['en', 'fr']);
	});
});

describe('the active catalog', () => {
	// The whole point of the store: components read `$t`, so changing the language has to
	// re-render them rather than requiring a restart.
	it('follows the locale store', () => {
		locale.set('fr');
		expect(get(t)).toBe(fr);
		expect(get(localeTag)).toBe('fr-FR');

		locale.set('en');
		expect(get(t)).toBe(en);
		expect(get(localeTag)).toBe('en-GB');
	});
});

describe('formatting a timestamp', () => {
	// The recovery prompt says when the spool was written; in a French session that has to be
	// a French date, which is the whole reason `locale.tag` exists.
	it('follows the interface language', () => {
		const stamp = Date.UTC(2026, 7, 27, 9, 30);
		const english = formatDateTime(stamp, 'en-GB');
		const french = formatDateTime(stamp, 'fr-FR');
		expect(english).not.toBe(french);
		expect(french).toMatch(/août/);
	});
});
