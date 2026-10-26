import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render } from '@testing-library/svelte';
import SupportLink from './SupportLink.svelte';
import { SUPPORT_URL } from './support';
import { mockTauri, type TauriMock } from './testing/tauriMock';

let core: TauriMock | null = null;

afterEach(async () => {
	cleanup();
	await core?.dispose();
	core = null;
	vi.restoreAllMocks();
});

describe('SupportLink', () => {
	// A link, named for what it does and saying where it goes, so Narrator reads it as one.
	it('is a link to the support page that says it leaves the app', () => {
		const link = render(SupportLink).getByRole('link');
		expect(link).toHaveAttribute('href', SUPPORT_URL);
		// jsdom's name computation drops the space a browser keeps before the hidden half.
		expect(link).toHaveAccessibleName(/^Buy me a coffee ?\(opens in your browser\)$/);
	});

	it('the page is the creator’s, over HTTPS', () => {
		expect(SUPPORT_URL).toBe('https://buymeacoffee.com/fmadore');
	});

	// WebView2 would open the href in a popup of its own, so in the app the core opens it.
	it('asks the core to open the page in the default browser', async () => {
		core = mockTauri();
		const open = vi.spyOn(window, 'open');
		await fireEvent.click(render(SupportLink).getByRole('link'));
		await vi.waitFor(() => expect(core!.callsTo('open_support_page')).toHaveLength(1));
		expect(open).not.toHaveBeenCalled();
	});

	it('says so when Windows will not open it', async () => {
		core = mockTauri();
		core.handle('open_support_page', () => {
			throw { id: 'error.openLink', detail: 'ShellExecute returned 2' };
		});
		const view = render(SupportLink);
		await fireEvent.click(view.getByRole('link'));
		expect(await view.findByRole('alert')).toHaveTextContent(
			'Windows could not open the page in your browser'
		);
	});

	// A browser preview has no core: a new tab is the closest thing to the default browser.
	it('opens a tab in a browser preview', async () => {
		const open = vi.spyOn(window, 'open').mockReturnValue(null);
		await fireEvent.click(render(SupportLink).getByRole('link'));
		expect(open).toHaveBeenCalledWith(SUPPORT_URL, '_blank', 'noopener,noreferrer');
	});
});
