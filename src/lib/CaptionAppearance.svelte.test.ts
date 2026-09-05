import { expect, it, vi } from 'vitest';
import { fireEvent, render, waitFor } from '@testing-library/svelte';
import CaptionAppearance from './CaptionAppearance.svelte';
import { createOverlayController } from './overlayController.svelte';
import { api } from './tauri';
import { overlayFontSize } from './stores';

it('keeps both appearances synchronized with separate accessible contrast descriptions', async () => {
	const setOverlayConfig = vi.fn().mockResolvedValue(undefined);
	const overlay = createOverlayController({ ...api, setOverlayConfig });
	overlayFontSize.set(42);
	const rail = render(CaptionAppearance, { heading: 'Rail', overlay });
	const settings = render(CaptionAppearance, { heading: 'Settings', overlay });
	const descriptions = Array.from(document.querySelectorAll('[aria-describedby]'));
	const ids = descriptions.map((node) => node.getAttribute('aria-describedby'));
	expect(new Set(ids).size).toBe(2);
	for (const id of ids) expect(document.getElementById(id!)).not.toBeNull();
	const increase = Array.from(rail.container.querySelectorAll('button')).find(
		(node) => node.textContent?.trim() === '+'
	)!;
	await fireEvent.click(increase);
	await waitFor(() => {
		expect(rail.container.querySelector('.stepper-value')?.textContent).toBe('44');
		expect(settings.container.querySelector('.stepper-value')?.textContent).toBe('44');
	});
	expect(setOverlayConfig).toHaveBeenCalledTimes(1);
	rail.unmount();
	settings.unmount();
});
