import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render } from '@testing-library/svelte';
import { flushSync } from 'svelte';
import ProviderConsentPrompt from './ProviderConsentPrompt.svelte';

function mount() {
	const onChoice = vi.fn();
	const view = render(ProviderConsentPrompt, { props: { vendor: 'Google Gemini', onChoice } });
	flushSync();
	return { ...view, onChoice };
}

describe('ProviderConsentPrompt', () => {
	it('is a labelled modal dialog naming the provider', () => {
		const dialog = mount().getByRole('dialog');
		expect(dialog).toHaveAttribute('aria-modal', 'true');
		expect(dialog).toHaveAccessibleName('Send audio to Google Gemini?');
	});

	// Review B8: the other decision prompts focus the safe answer, and so does this one. Enter on
	// the prompt as it opens sends nothing.
	it('puts focus on Cancel, not on Send', () => {
		const { getByRole } = mount();
		expect(getByRole('button', { name: 'Cancel' })).toHaveFocus();
		expect(getByRole('button', { name: 'Send to Google Gemini' })).not.toHaveFocus();
	});

	it('reports Send and Cancel', async () => {
		const { getByRole, onChoice } = mount();
		await fireEvent.click(getByRole('button', { name: 'Send to Google Gemini' }));
		expect(onChoice).toHaveBeenLastCalledWith(true);
		await fireEvent.click(getByRole('button', { name: 'Cancel' }));
		expect(onChoice).toHaveBeenLastCalledWith(false);
	});
});
