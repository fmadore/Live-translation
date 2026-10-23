import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { fireEvent, render, waitFor } from '@testing-library/svelte';
import OverlayMoveButton from './OverlayMoveButton.svelte';
import { createOverlayController } from './overlayController.svelte';
import { operatorContextMap } from './operatorContext';
import { api } from './tauri';
import { setLocale } from './i18n';
import { overlayPlaced } from './stores';

// S6: one button for the rail, the pre-flight row and Settings, so the three cannot drift apart.

function controller() {
	return createOverlayController({
		...api,
		showOverlay: vi.fn().mockResolvedValue(undefined),
		setOverlayClickThrough: vi.fn().mockResolvedValue(undefined),
		setOverlayConfig: vi.fn().mockResolvedValue(undefined)
	});
}

function mount(props: Record<string, unknown>) {
	return render(OverlayMoveButton, {
		props: { wording: 'place', ...props },
		context: operatorContextMap({ overlay: controller() })
	});
}

beforeEach(() => {
	setLocale('en');
	overlayPlaced.set(false);
});
afterEach(() => overlayPlaced.set(false));

it('enters move mode and names the way out, as a plain button rather than a pressed toggle', async () => {
	const view = mount({ wording: 'move' });
	const move = view.getByRole('button', { name: 'Move overlay' });
	expect(move).not.toHaveAttribute('aria-pressed');
	expect(move).not.toHaveClass('primary');
	await fireEvent.click(move);
	const done = await waitFor(() => view.getByRole('button', { name: 'Finish moving the overlay' }));
	expect(done).toHaveTextContent('Done');
	expect(done).toHaveClass('primary');
	expect(done).not.toHaveAttribute('aria-pressed');
});

it('words placing in the pre-flight and Settings terms, and draws itself as asked while idle', async () => {
	const view = mount({ idleVariant: 'warn', size: 'sm', icon: false });
	const place = view.getByRole('button', { name: 'Move overlay' });
	expect(place).toHaveClass('warn', 'sm');
	expect(place.querySelector('svg')).toBeNull();
	await fireEvent.click(place);
	expect(
		await waitFor(() => view.getByRole('button', { name: 'Finish placing the overlay' }))
	).toHaveClass('primary');
});

it('stays out of reach where there is no overlay to move', () => {
	const view = mount({ wide: true, disabled: true });
	const button = view.getByRole('button', { name: 'Move overlay' });
	expect(button).toBeDisabled();
	expect(button).toHaveClass('wide');
	expect(button.querySelector('svg')).not.toBeNull();
});

it('says what is missing when mounted without the page’s context', () => {
	expect(() => render(OverlayMoveButton, { props: { wording: 'move' } })).toThrow(
		'No overlay controller in context'
	);
});
