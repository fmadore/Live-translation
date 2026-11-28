import { afterEach, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render } from '@testing-library/svelte';
import { get } from 'svelte/store';
import SettingsDialog from './SettingsDialog.svelte';
import { setLocale } from './i18n';
import { TEXT_SIZE_KEY, textSize } from './textSize';

afterEach(() => {
	cleanup();
	textSize.set(100);
	setLocale('en');
});

// The corner × is the first control in the panel, and opening Settings to close it again is
// never the reason it was opened: focus starts on the tab it opens to, from where the arrows
// move between tabs and Tab reaches the content.
it('opens with focus on the selected tab, not the corner Close', () => {
	setLocale('en');
	const view = render(SettingsDialog, {
		props: { browserMode: true, tab: 'app', onHideWindow: vi.fn(), onClose: vi.fn() }
	});
	const app = view.getByRole('tab', { name: 'App' });
	expect(app).toHaveAttribute('aria-selected', 'true');
	expect(app).toHaveFocus();
});

// Issue #283: the operator window's own text size, a percentage of the system's. 100% is the
// default and is marked as such, rather than a separate "System" entry that would draw the
// same window; each choice is written the way the interface language writes a percentage.
it('offers the Text size choices and keeps the one picked', async () => {
	setLocale('en');
	const view = render(SettingsDialog, {
		props: { browserMode: true, tab: 'app', onHideWindow: vi.fn(), onClose: vi.fn() }
	});
	const select = view.getByRole('combobox', { name: 'Text size' });
	expect(select).toHaveAccessibleDescription(/on top of the text size set in Windows/);
	expect([...select.querySelectorAll('option')].map((o) => o.textContent?.trim())).toEqual([
		'90%',
		'100% (default)',
		'115%',
		'130%',
		'150%'
	]);
	expect(select).toHaveValue('100');

	await fireEvent.change(select, { target: { value: '130' } });
	expect(get(textSize)).toBe(130);
	expect(localStorage.getItem(TEXT_SIZE_KEY)).toBe('130');
});

it('writes the Text size percentages as French does', () => {
	setLocale('fr');
	const view = render(SettingsDialog, {
		props: { browserMode: true, tab: 'app', onHideWindow: vi.fn(), onClose: vi.fn() }
	});
	const select = view.getByRole('combobox', { name: 'Taille du texte' });
	// A no-break space before the sign, narrow or not depending on the engine's ICU data.
	expect(select.querySelector('option')?.textContent?.trim()).toMatch(/^90[  ]%$/);
});
