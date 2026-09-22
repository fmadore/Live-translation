import { expect, it, vi } from 'vitest';
import { fireEvent, render, waitFor } from '@testing-library/svelte';
import LanguagePicker from './LanguagePicker.svelte';
import { locale } from './i18n';

const base = {
	value: 'en' as const,
	provider: 'gemini' as const,
	favourites: ['en', 'fr'] as ('en' | 'fr')[],
	onchange: vi.fn(),
	onpin: vi.fn()
};

it('opens on the current value, filters and selects with Enter, and restores on Escape', async () => {
	locale.set('en');
	const onchange = vi.fn();
	const view = render(LanguagePicker, { ...base, onchange });
	const input = view.getByRole('combobox') as HTMLInputElement;
	input.focus();
	await waitFor(() => expect(input).toHaveAttribute('aria-expanded', 'true'));
	expect(input.selectionEnd).toBe(input.value.length);
	await fireEvent.input(input, { target: { value: 'francais' } });
	await waitFor(() => expect(view.getAllByRole('option')).toHaveLength(1));
	expect(
		document.getElementById(input.getAttribute('aria-activedescendant')!)?.textContent
	).toContain('French');
	await fireEvent.keyDown(input, { key: 'Enter' });
	expect(onchange).toHaveBeenCalledWith('fr');
	expect(input).toHaveAttribute('aria-expanded', 'false');
	await fireEvent.keyDown(input, { key: 'ArrowDown' });
	await fireEvent.input(input, { target: { value: 'nonsense' } });
	expect(view.getByRole('status')).toHaveTextContent('No matching languages');
	await fireEvent.keyDown(input, { key: 'Escape' });
	expect(input.value).toBe('EN · English');
	view.unmount();
});

it('supports arrows, Home/End, a keyboard pin button, and blur without a focus trap', async () => {
	const onpin = vi.fn();
	const view = render(LanguagePicker, { ...base, onpin });
	const input = view.getByRole('combobox');
	input.focus();
	await fireEvent.keyDown(input, { key: 'ArrowDown' });
	await fireEvent.keyDown(input, { key: 'End' });
	const options = view.getAllByRole('option');
	expect(input.getAttribute('aria-activedescendant')).toBe(options.at(-1)?.id);
	await fireEvent.keyDown(input, { key: 'Home' });
	expect(input.getAttribute('aria-activedescendant')).toBe(options[0].id);
	const pin = view.getByRole('button', { name: 'Unpin English' });
	expect(pin).toHaveAttribute('tabindex', '0');
	pin.focus();
	await fireEvent.keyDown(pin, { key: 'Escape' });
	expect(input).toHaveAttribute('aria-expanded', 'false');
	await fireEvent.keyDown(input, { key: 'ArrowDown' });
	await fireEvent.click(view.getByRole('button', { name: 'Unpin English' }));
	expect(onpin).toHaveBeenCalledWith('en');
	await fireEvent.focusOut(input, { relatedTarget: document.body });
	expect(input).toHaveAttribute('aria-expanded', 'false');
	view.unmount();
});

it('keeps unsupported favourites visible with a reason and refuses selection', async () => {
	const onchange = vi.fn();
	const view = render(LanguagePicker, {
		...base,
		provider: 'openai',
		favourites: ['sw', 'en'],
		value: 'sw',
		onchange,
		error: 'OpenAI Realtime Translate does not support Swahili — choose another language.'
	});
	const input = view.getByRole('combobox');
	input.focus();
	await waitFor(() =>
		expect(view.getByRole('option', { name: /Swahili/ })).toHaveAttribute('aria-disabled', 'true')
	);
	await fireEvent.keyDown(input, { key: 'Enter' });
	expect(onchange).not.toHaveBeenCalled();
	expect(input).toHaveAttribute('aria-invalid', 'true');
	expect(view.getByRole('status')).toHaveTextContent('choose another language');
	view.unmount();
});
