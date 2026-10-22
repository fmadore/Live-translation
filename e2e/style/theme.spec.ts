// The theme setting in Edge, the engine WebView2 embeds: the first frame, which `src/app.html`
// paints before the app has loaded, and the native title bar, which `src/lib/theme.ts` hands
// back to Windows while the setting is "System". The colours themselves are in the light
// snapshots `computed.spec.ts` records.

import { expect, test } from '@playwright/test';
import { DEFAULT_WINDOW, openOperator } from './operator';

test.use({ viewport: DEFAULT_WINDOW, colorScheme: 'light' });

test('a light desktop gets a light first frame, before the app has loaded', async ({ page }) => {
	// No script but the inline one in app.html runs, so whatever is on screen is the first
	// frame a light desktop would see while the app's modules are still loading.
	await page.route('**/_app/**/*.js', (route) => route.abort());
	await page.goto('/');
	await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
	// `--surface-0` of the light block, not the dark ground.
	await expect(page.locator('body')).toHaveCSS('background-color', 'rgb(243, 244, 246)');
});

test('"System" follows the desktop and leaves the title bar to Windows', async ({ page }) => {
	const { core } = await openOperator(page, { state: 'idle-whisper', locale: 'en', scale: 1 });
	await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
	expect(core.callsTo('plugin:window|set_theme')).toEqual([{ label: 'operator', value: null }]);
});

test('the overlay keeps the dark tokens on a light desktop', async ({ page }) => {
	await page.addInitScript(() => localStorage.setItem('appearance.theme', 'light'));
	await page.goto('/overlay/');
	await expect(page.locator('html')).not.toHaveAttribute('data-theme');
	expect(await page.evaluate(() => getComputedStyle(document.documentElement).colorScheme)).toBe(
		'dark'
	);
});
