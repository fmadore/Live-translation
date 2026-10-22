// Computed-style snapshots of the operator window at its default size: every state in English
// at 100%, and each state again at the corners of the matrix where the values move — French
// and German at 100% (their own markup and labels), English at 225% (every size and `em`) —
// and once more in the light theme, in English at 100% (only colours change there).
//
// A snapshot is one line per element: its path, then the CSS values it computes, without
// geometry. A failure opens with each distinct change and the elements it reached, e.g.
// `padding: 12px → 16px — 4 element(s)`, followed by Playwright's line diff. Regenerate after
// an intended change with `npm run test:style -- --update-snapshots`, and read the diff.

import { existsSync, readFileSync } from 'node:fs';
import { expect, test } from '@playwright/test';
import {
	DEFAULT_WINDOW,
	openOperator,
	scopeOf,
	STATE_NAMES,
	type Locale,
	type Scale,
	type Theme
} from './operator';
import { computedStyles } from './probe';
import { describeChanges } from './styleDiff';

const CORNERS: { locale: Locale; scale: Scale; theme?: Theme }[] = [
	{ locale: 'en', scale: 1 },
	{ locale: 'fr', scale: 1 },
	{ locale: 'de', scale: 1 },
	{ locale: 'en', scale: 2.25 },
	{ locale: 'en', scale: 1, theme: 'light' }
];

test.use({ viewport: DEFAULT_WINDOW });

for (const { locale, scale, theme } of CORNERS) {
	const percent = Math.round(scale * 100);
	const suffix = theme ? `-${theme}` : '';
	test.describe(`${locale} at ${percent}%${theme ? ` · ${theme}` : ''}`, () => {
		for (const state of STATE_NAMES) {
			test(state, async ({ page }) => {
				await openOperator(page, { state, locale, scale, theme });
				const scope = scopeOf(state);
				const header = [
					`# ${state} · ${locale} · ${percent}%${theme ? ` · ${theme}` : ''} · ` +
						`${DEFAULT_WINDOW.width}×${DEFAULT_WINDOW.height}` +
						(scope ? ` · ${scope}` : ''),
					'# path { property: computed value } — inherited properties where they differ from',
					'# the parent, the rest where they differ from the same element unstyled.'
				];
				const snapshot = [...header, ...(await computedStyles(page, scope)), ''].join('\n');

				const name = `${state}-${locale}-${percent}${suffix}.txt`;
				const committed = test.info().snapshotPath(name);
				const changes = existsSync(committed)
					? describeChanges(readFileSync(committed, 'utf8'), snapshot)
					: '';
				expect(snapshot, changes || undefined).toMatchSnapshot(name);
			});
		}
	});
}
