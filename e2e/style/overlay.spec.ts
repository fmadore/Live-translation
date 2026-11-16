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

// A notice for a break replaces the captions with one line, and shrinks, as a row does, until it
// is inside a shallow strip. `?notice=` stands in for the operator's push in the preview.
for (const height of [160, 72]) {
	test(`a break notice is whole inside a ${height} px strip`, async ({ page }) => {
		await page.setViewportSize({ width: 1280, height });
		const text = 'Gleich geht es weiter – wir setzen um 14:30 Uhr mit dem zweiten Vortrag fort';
		await page.goto(`/overlay/?language=bilingual-both&notice=${encodeURIComponent(text)}`);
		const line = page.locator('.captions.notice .line:not(.probe)');
		await expect(line).toHaveText(text);
		// No caption row is drawn under it.
		await expect(page.locator('.captions .texts')).toHaveCount(0);
		await expect
			.poll(() => line.evaluate((el) => el.getBoundingClientRect().bottom))
			.toBeLessThanOrEqual(height);
		const { top, scrollHeight, clientHeight } = await line.evaluate((el) => ({
			top: el.getBoundingClientRect().top,
			scrollHeight: el.parentElement!.scrollHeight,
			clientHeight: el.parentElement!.clientHeight
		}));
		expect(top).toBeGreaterThanOrEqual(0);
		expect(scrollHeight).toBeLessThanOrEqual(clientHeight);
	});
}
