import { expect, it, vi } from 'vitest';
import { fireEvent, render, waitFor } from '@testing-library/svelte';
import CaptionAppearance from './CaptionAppearance.svelte';
import { createOverlayController } from './overlayController.svelte';
import { api } from './tauri';
import { overlayFontSize, overlayCaptionLayout, overlayCaptionWidth } from './stores';
import { CAPTION_LAYOUT_KEY, loadCaptionLayout } from './captionLayout';

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

it('switches layout live, remembers compact width, and resets to fit window', async () => {
	const setOverlayConfig = vi.fn().mockResolvedValue(undefined);
	const overlay = createOverlayController({ ...api, setOverlayConfig });
	overlayCaptionLayout.set('fit');
	overlayCaptionWidth.set(44);
	const view = render(CaptionAppearance, { heading: 'Appearance', overlay });
	const selector = view.container.querySelector('select')!;
	expect(view.container.querySelectorAll('.stepper')).toHaveLength(2);
	await fireEvent.change(selector, { target: { value: 'compact' } });
	expect(view.container.querySelectorAll('.stepper')).toHaveLength(3);
	expect(setOverlayConfig).toHaveBeenLastCalledWith(
		expect.objectContaining({ captionLayout: 'compact', captionWidth: 44 })
	);
	expect(localStorage.getItem(CAPTION_LAYOUT_KEY)).toBe('compact');
	expect(loadCaptionLayout()).toBe('compact');
	await fireEvent.change(selector, { target: { value: 'fit' } });
	await fireEvent.change(selector, { target: { value: 'compact' } });
	expect(setOverlayConfig).toHaveBeenLastCalledWith(expect.objectContaining({ captionWidth: 44 }));
	overlay.resetOverlayAppearance();
	await waitFor(() => expect(selector.value).toBe('fit'));
	expect(loadCaptionLayout()).toBe('fit');
	view.unmount();
});
