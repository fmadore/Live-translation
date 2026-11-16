import { afterEach, describe, expect, it } from 'vitest';
import { cleanup, fireEvent, render, within } from '@testing-library/svelte';
import UpdateNotice from './UpdateNotice.svelte';
import UpdateSettings from './UpdateSettings.svelte';
import { createUpdateCheck } from './updateCheck.svelte';
import { mockTauri, type TauriMock } from './testing/tauriMock';
import type { UpdateStatus } from './types';

let core: TauriMock | null = null;

afterEach(async () => {
	cleanup();
	await core?.dispose();
	core = null;
});

const BASE: UpdateStatus = {
	eligible: true,
	checks: 'on',
	ask: false,
	lastCheckedMs: null,
	available: null
};

/** A core whose `update_status` answers `status`, and a controller that has asked it. */
async function loaded(status: Partial<UpdateStatus>) {
	core = mockTauri();
	let current: UpdateStatus = { ...BASE, ...status };
	core.handle('update_status', () => current);
	core.handle('skip_update_version', () => (current = { ...current, available: null }));
	core.handle(
		'set_update_checks',
		({ on }) => (current = { ...current, ask: false, checks: on ? 'on' : 'off' })
	);
	core.handle('check_for_update', () => current);
	const updates = createUpdateCheck();
	await updates.load();
	return updates;
}

describe('UpdateNotice on the start screen', () => {
	it('says which version is available, in a live region, with its two answers', async () => {
		const updates = await loaded({ available: '1.15.0' });
		const view = render(UpdateNotice, { updates });

		expect(view.getByRole('status')).toHaveTextContent('Version 1.15.0 is available');
		expect(view.getByRole('button', { name: /^Release notes ?\(opens in your browser\)$/ }));
		expect(view.getByRole('button', { name: 'Not now' }));
	});

	it('opens the release page through the core, which builds its address', async () => {
		const updates = await loaded({ available: '1.15.0' });
		const view = render(UpdateNotice, { updates });

		await fireEvent.click(view.getByRole('button', { name: /^Release notes/ }));

		await expect.poll(() => core!.callsTo('open_release_page')).toHaveLength(1);
	});

	it('says so when the page cannot be opened', async () => {
		const updates = await loaded({ available: '1.15.0' });
		core!.handle('open_release_page', () => {
			throw { id: 'error.openLink', detail: 'ShellExecute returned 2' };
		});
		const view = render(UpdateNotice, { updates });

		await fireEvent.click(view.getByRole('button', { name: /^Release notes/ }));

		expect(await view.findByRole('alert')).toHaveTextContent('could not open the page');
	});

	it('Not now hides the notice and leaves the empty region in place', async () => {
		const updates = await loaded({ available: '1.15.0' });
		const view = render(UpdateNotice, { updates });

		await fireEvent.click(view.getByRole('button', { name: 'Not now' }));

		await expect.poll(() => view.getByRole('status').textContent).toBe('');
		expect(view.queryByRole('button')).toBeNull();
		expect(core!.callsTo('skip_update_version')).toHaveLength(1);
	});

	it('asks from the second launch on, until answered', async () => {
		const updates = await loaded({ checks: 'unanswered', ask: true });
		const view = render(UpdateNotice, { updates });

		expect(view.getByRole('status')).toHaveTextContent(
			'Check for new versions? This asks GitHub once a day.'
		);
		await fireEvent.click(view.getByRole('button', { name: 'Turn on' }));

		await expect.poll(() => core!.callsTo('set_update_checks')).toEqual([{ on: true }]);
		await expect.poll(() => view.queryByRole('button', { name: 'Turn on' })).toBeNull();
		expect(core!.callsTo('check_for_update')).toHaveLength(1);
	});

	it('No thanks answers without a check', async () => {
		const updates = await loaded({ checks: 'unanswered', ask: true });
		const view = render(UpdateNotice, { updates });

		await fireEvent.click(view.getByRole('button', { name: 'No thanks' }));

		await expect.poll(() => core!.callsTo('set_update_checks')).toEqual([{ on: false }]);
		await expect.poll(() => view.queryByRole('button', { name: 'No thanks' })).toBeNull();
		expect(core!.callsTo('check_for_update')).toHaveLength(0);
	});

	it('shows nothing at all on a copy that does not check', async () => {
		const updates = await loaded({ eligible: false, available: '1.15.0', ask: true });
		const view = render(UpdateNotice, { updates });
		expect(view.container.querySelector('*')).toBeNull();
	});

	it('shows nothing in a browser preview, where there is no core to ask', async () => {
		const updates = createUpdateCheck();
		await updates.load();
		const view = render(UpdateNotice, { updates });
		expect(view.container.querySelector('*')).toBeNull();
	});
});

describe('UpdateSettings in Settings › App', () => {
	it('has the switch, the notice and the last check', async () => {
		const updates = await loaded({ available: '1.15.0', lastCheckedMs: Date.UTC(2026, 10, 16) });
		const view = render(UpdateSettings, { updates });

		expect(view.getByRole('heading', { name: 'Updates' }));
		expect(view.getByRole('checkbox', { name: 'Check for new versions' })).toBeChecked();
		expect(view.getByText('Version 1.15.0 is available'));
		// Not a live region here: Settings is opened to read, not to be told.
		expect(view.queryByRole('status')).toBeNull();
		expect(view.getByText(/^Last checked: /)).toHaveTextContent('2026');
	});

	it('turns the check off with the switch', async () => {
		const updates = await loaded({});
		const view = render(UpdateSettings, { updates });

		await fireEvent.click(view.getByRole('checkbox', { name: 'Check for new versions' }));

		await expect.poll(() => core!.callsTo('set_update_checks')).toEqual([{ on: false }]);
		await expect
			.poll(() => view.getByRole('checkbox', { name: 'Check for new versions' }))
			.not.toBeChecked();
		expect(view.getByText('Not checked yet'));
	});

	it('does not ask the start screen’s question', async () => {
		const updates = await loaded({ checks: 'unanswered', ask: true });
		const view = render(UpdateSettings, { updates });
		expect(view.getByRole('checkbox', { name: 'Check for new versions' })).not.toBeChecked();
		expect(within(view.container).queryByRole('button', { name: 'Turn on' })).toBeNull();
	});

	it('is absent on a copy that does not check', async () => {
		const updates = await loaded({ eligible: false });
		const view = render(UpdateSettings, { updates });
		expect(view.container.querySelector('*')).toBeNull();
	});
});
