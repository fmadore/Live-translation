import { afterEach, expect, it, vi } from 'vitest';
import { cleanup, render } from '@testing-library/svelte';
import SettingsDialog from './SettingsDialog.svelte';
import { setLocale } from './i18n';

afterEach(() => cleanup());

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
