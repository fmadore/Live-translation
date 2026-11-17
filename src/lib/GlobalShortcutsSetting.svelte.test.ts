import { afterEach, describe, expect, it } from 'vitest';
import { cleanup, fireEvent, render } from '@testing-library/svelte';
import GlobalShortcutsSetting from './GlobalShortcutsSetting.svelte';
import { globalShortcutStatus } from './globalShortcuts';
import { globalShortcuts } from './stores';
import { setLocale } from './i18n';

afterEach(() => {
	cleanup();
	globalShortcuts.set(false);
	globalShortcutStatus.set({ registered: [], failed: [] });
	setLocale('en');
});

const SWITCH = 'Use shortcuts while other apps are in front';

describe('GlobalShortcutsSetting', () => {
	it('is off by default and lists the two keys, with an empty status region', () => {
		const view = render(GlobalShortcutsSetting, { browserMode: false });
		const toggle = view.getByRole('checkbox', { name: SWITCH });
		expect(toggle).not.toBeChecked();
		expect(view.getByText('Ctrl+Alt+Shift+P', { exact: false }));
		expect(view.getByText('Pause / resume captions'));
		expect(view.getByText('Show / hide overlay'));
		expect(view.getByRole('status')).toHaveTextContent('');
		// Two keys, and never a Start or Stop one.
		expect(view.container.querySelectorAll('dt')).toHaveLength(2);
		expect(view.queryByText('Start / stop session')).toBeNull();
	});

	it('persists the switch', async () => {
		const view = render(GlobalShortcutsSetting, { browserMode: false });
		await fireEvent.click(view.getByRole('checkbox', { name: SWITCH }));
		expect(localStorage.getItem('shortcuts.global')).toBe('true');
	});

	it('names a key another app holds, in a status region, in the interface language', async () => {
		globalShortcuts.set(true);
		globalShortcutStatus.set({ registered: ['toggle-pause'], failed: ['toggle-overlay'] });
		const view = render(GlobalShortcutsSetting, { browserMode: false });
		expect(view.getByRole('status')).toHaveTextContent(
			'Could not register Ctrl+Alt+Shift+O: another app already uses it.'
		);
		setLocale('fr');
		await expect
			.poll(() => view.getByRole('status').textContent)
			.toContain('Impossible d’enregistrer Ctrl+Alt+Maj+O');
	});

	it('says nothing of a refusal once the switch is off', () => {
		globalShortcutStatus.set({ registered: [], failed: ['toggle-pause'] });
		const view = render(GlobalShortcutsSetting, { browserMode: false });
		expect(view.getByRole('status')).toHaveTextContent('');
	});

	it('cannot be switched on in a browser preview', () => {
		const view = render(GlobalShortcutsSetting, { browserMode: true });
		expect(view.getByRole('checkbox', { name: SWITCH })).toBeDisabled();
	});
});
