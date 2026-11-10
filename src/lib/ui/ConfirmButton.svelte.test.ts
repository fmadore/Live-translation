import { fireEvent, render } from '@testing-library/svelte';
import { expect, it, vi } from 'vitest';
import ConfirmButton from './ConfirmButton.svelte';

function setup(extra: { disabled?: boolean; cancelDisabled?: boolean } = {}) {
	const onask = vi.fn();
	const onconfirm = vi.fn();
	const oncancel = vi.fn();
	const props = {
		confirming: false,
		label: 'Delete',
		confirmLabel: 'Delete permanently',
		cancelLabel: 'Cancel',
		onask,
		onconfirm,
		oncancel,
		...extra
	};
	const view = render(ConfirmButton, props);
	return { view, props, onask, onconfirm, oncancel };
}

it('asks on the first press and acts only on the second', async () => {
	const { view, props, onask, onconfirm } = setup();
	const button = view.getByRole('button', { name: 'Delete' });
	expect(view.queryByRole('button', { name: 'Cancel' })).toBeNull();
	await fireEvent.click(button);
	expect(onask).toHaveBeenCalledOnce();
	expect(onconfirm).not.toHaveBeenCalled();
	await view.rerender({ ...props, confirming: true });
	// The same element takes the confirmation's name, so focus and the keyboard stay on it.
	expect(view.getByRole('button', { name: 'Delete permanently' })).toBe(button);
	await fireEvent.click(button);
	expect(onconfirm).toHaveBeenCalledOnce();
	expect(onask).toHaveBeenCalledOnce();
});

it('keeps focus on the button as it turns into the question', async () => {
	const { view, props } = setup();
	const button = view.getByRole('button', { name: 'Delete' });
	button.focus();
	await fireEvent.click(button);
	await view.rerender({ ...props, confirming: true });
	expect(button).toHaveFocus();
});

it('offers Cancel while asking, and withdraws the question through the caller', async () => {
	const { view, props, oncancel, onconfirm } = setup();
	await view.rerender({ ...props, confirming: true });
	await fireEvent.click(view.getByRole('button', { name: 'Cancel' }));
	expect(oncancel).toHaveBeenCalledOnce();
	expect(onconfirm).not.toHaveBeenCalled();
	await view.rerender({ ...props, confirming: false });
	expect(view.queryByRole('button', { name: 'Cancel' })).toBeNull();
	expect(view.getByRole('button', { name: 'Delete' })).toBeInTheDocument();
});

it('disables Cancel with the button unless told otherwise', async () => {
	const { view, props } = setup({ disabled: true });
	await view.rerender({ ...props, confirming: true });
	expect(view.getByRole('button', { name: 'Delete permanently' })).toBeDisabled();
	expect(view.getByRole('button', { name: 'Cancel' })).toBeDisabled();
	await view.rerender({ ...props, confirming: true, cancelDisabled: false });
	expect(view.getByRole('button', { name: 'Cancel' })).toBeEnabled();
});
