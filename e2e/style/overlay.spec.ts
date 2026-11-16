// The overlay's caption rows in Edge, the engine WebView2 embeds, at the default 160 px strip.
// Both speakers captioned in two languages make four rows, which that strip has no room for at
// the chosen size: in 1.14.0 each row showed its label and no caption (D1). The browser
// preview has no core, so `?language=bilingual-both` supplies the four rows.

import { expect, test, type Page } from '@playwright/test';

test.use({ viewport: { width: 1280, height: 160 } });

/** Each row's caption text, without its label or the hidden measuring probe. */
async function rowTexts(page: Page) {
	const lines = page.locator('.captions .row .texts > .text-region > .line:not(.probe)');
	await expect(lines).toHaveCount(4);
	return lines.allTextContents();
}

for (const original of [false, true]) {
	test(`four rows each show a caption in the default strip${original ? ' with the original on' : ''}`, async ({
		page
	}) => {
		if (original)
			await page.addInitScript(() => localStorage.setItem('overlay.showOriginal', 'true'));
		await page.goto('/overlay/?language=bilingual-both');
		await expect
			.poll(async () => (await rowTexts(page)).filter((text) => text.trim()).length)
			.toBe(4);
		// The translation comes first: with no room under four rows, the original gives way.
		await expect(page.locator('.original')).toHaveCount(0);
		// Every row's text is inside the window, not clipped below it.
		const bottoms = await page
			.locator('.captions .row')
			.evaluateAll((rows) => rows.map((row) => row.getBoundingClientRect().bottom));
		for (const bottom of bottoms) expect(bottom).toBeLessThanOrEqual(160);
	});
}
