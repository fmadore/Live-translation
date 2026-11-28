// Horizontal overflow across the whole matrix — every state, in English, French and German, at
// 100, 150 and 225% text, and at the 90% Text size — at the operator window's minimum size,
// where long German and French labels have the least room. Any element whose content is wider
// than its box fails the run, unless it is listed below as meant.

import { expect, test } from '@playwright/test';
import {
	LOCALES,
	MINIMUM_WINDOW,
	openOperator,
	OVERFLOW_STATE_NAMES,
	SCALES,
	SMALLEST_TEXT_SIZE,
	type Scale
} from './operator';
import { overflow, type Offender } from './probe';

/** Overflows that are part of the design. Each says why, and matches as narrowly as it can:
 *  an entry here must never be able to hide clipped text. */
const INTENDED: { reason: string; matches(o: Offender): boolean }[] = [
	{
		reason:
			"A dialog's close button is pulled 6px into the corner with a negative margin " +
			'(`.header :global(.dismiss)` in ModalPrompt.svelte), into the padding of the dialog, ' +
			'so it does not make the header row taller.',
		matches: (o) =>
			/\[role=dialog\] "[^"]*" › div\.header$/.test(o.path) && o.scrollWidth - o.clientWidth <= 6
	},
	{
		reason:
			"A level meter's peak marker rides the edge of a full-width carrier that is translated " +
			'across the track by the peak level, and the track clips it (LevelMeter.svelte). The ' +
			'track holds no text.',
		matches: (o) => /› div\.track\[role=meter\] "[^"]*"$/.test(o.path) && o.text === ''
	}
];

function describe(offenders: Offender[]): string {
	return [
		`${offenders.length} element(s) overflow horizontally:`,
		...offenders.map(
			(o) =>
				`  ${o.path}\n    "${o.text}"\n    content ${o.scrollWidth}px in a ${o.clientWidth}px box ` +
				`(overflow-x: ${o.overflowX})`
		)
	].join('\n');
}

test.use({ viewport: MINIMUM_WINDOW });

/** Windows' slider at each step, and the smallest Settings › App › Text size on a system at
 *  100% — the one way the window's text gets smaller than the slider allows (issue #283). */
const SIZES: { label: string; scale: Scale; textSize?: number }[] = [
	...SCALES.map((scale) => ({ label: `${scale * 100}%`, scale })),
	{ label: `${SMALLEST_TEXT_SIZE}% (Text size)`, scale: 1, textSize: SMALLEST_TEXT_SIZE }
];

for (const locale of LOCALES) {
	for (const { label, scale, textSize } of SIZES) {
		test.describe(`${locale} at ${label}`, () => {
			for (const state of OVERFLOW_STATE_NAMES) {
				test(state, async ({ page }) => {
					await openOperator(page, { state, locale, scale, textSize });
					const unexpected = (await overflow(page)).filter(
						(o) => !INTENDED.some((intended) => intended.matches(o))
					);
					expect(unexpected, describe(unexpected)).toEqual([]);
				});
			}
		});
	}
}
