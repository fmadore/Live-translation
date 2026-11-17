import { expect, it, vi } from 'vitest';
import { fireEvent, render } from '@testing-library/svelte';
import OverlayMoveChrome from './OverlayMoveChrome.svelte';
import { OVERLAY_FONT_MAX, OVERLAY_FONT_MIN } from '#lib/appearance.js';

function chrome(props: { fontSize?: number; width?: number; height?: number } = {}) {
	const onBump = vi.fn();
	const onSnap = vi.fn();
	const onLock = vi.fn();
	const view = render(OverlayMoveChrome, {
		fontSize: 38,
		width: 1280,
		height: 160,
		onBump,
		onSnap,
		onLock,
		...props
	});
	return { view, onBump, onSnap, onLock };
}

it('reads out the window size and the caption size', () => {
	const { view } = chrome();
	expect(view.getByText('1280 × 160')).toBeTruthy();
	expect(view.getByRole('group', { name: 'Size' }).querySelector('output')?.textContent).toBe(
		'38 px'
	);
	// A short strip has no room for the placeholder under the toolbar.
	expect(view.queryByText(/Captions will sit here/)).toBeNull();
});

it('shows the placeholder at the chosen size in a tall region', () => {
	const { view } = chrome({ height: 400 });
	expect(view.getByText('Captions will sit here, two lines at 38 px.')).toBeTruthy();
});

it('bumps the size by a step either way', async () => {
	const { view, onBump } = chrome();
	await fireEvent.click(view.getByRole('button', { name: 'Larger captions' }));
	await fireEvent.click(view.getByRole('button', { name: 'Smaller captions' }));
	expect(onBump.mock.calls).toEqual([[2], [-2]]);
});

it('stops at the bounds of the caption size', () => {
	const small = chrome({ fontSize: OVERLAY_FONT_MIN });
	expect(small.view.getByRole('button', { name: 'Smaller captions' })).toBeDisabled();
	small.view.unmount();
	const large = chrome({ fontSize: OVERLAY_FONT_MAX });
	expect(large.view.getByRole('button', { name: 'Larger captions' })).toBeDisabled();
});

it('snaps to the bottom and locks into place', async () => {
	const { view, onSnap, onLock } = chrome();
	await fireEvent.click(view.getByRole('button', { name: 'Snap to bottom' }));
	expect(onSnap).toHaveBeenCalledOnce();
	expect(onLock).not.toHaveBeenCalled();
	await fireEvent.click(view.getByRole('button', { name: 'Lock into place' }));
	expect(onLock).toHaveBeenCalledOnce();
});
