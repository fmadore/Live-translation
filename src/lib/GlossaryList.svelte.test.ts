import { get } from 'svelte/store';
import { afterEach, beforeEach, expect, it } from 'vitest';
import { fireEvent, render, waitFor } from '@testing-library/svelte';
import GlossaryList from './GlossaryList.svelte';
import { options } from './stores';
import { DEFAULT_START_OPTIONS, SESSION_OPTIONS_KEY } from './startOptions';
import { MAX_GLOSSARY_TERMS } from './glossary';
import { setLocale } from './i18n';

const VOXTRAL = { ...DEFAULT_START_OPTIONS, provider: 'mistral' } as const;

beforeEach(() => {
	options.set({ ...VOXTRAL, glossary: ['UNESCO', 'Ki-Zerbo'] });
});

afterEach(() => {
	options.set({ ...DEFAULT_START_OPTIONS });
	setLocale('en');
});

it('adds and removes names and terms in the saved setup', async () => {
	const view = render(GlossaryList, { locked: false });
	const field = view.getByLabelText('Add a name or term');

	await fireEvent.input(field, { target: { value: '  Université Joseph Ki-Zerbo ' } });
	await fireEvent.click(view.getByRole('button', { name: 'Add' }));
	expect(get(options).glossary).toEqual(['UNESCO', 'Ki-Zerbo', 'Université Joseph Ki-Zerbo']);
	// Saved with the rest of the setup, so the next launch and a profile both have it.
	expect(JSON.parse(localStorage.getItem(SESSION_OPTIONS_KEY)!).glossary).toEqual([
		'UNESCO',
		'Ki-Zerbo',
		'Université Joseph Ki-Zerbo'
	]);
	expect(field).toHaveValue('');
	await waitFor(() =>
		expect(view.getByRole('status')).toHaveTextContent('“Université Joseph Ki-Zerbo” added.')
	);

	await fireEvent.click(view.getByRole('button', { name: 'Remove “UNESCO”' }));
	expect(get(options).glossary).toEqual(['Ki-Zerbo', 'Université Joseph Ki-Zerbo']);
	// Focus stays in the list, on the term that took the removed one's place.
	await waitFor(() =>
		expect(view.getByRole('button', { name: 'Remove “Ki-Zerbo”' })).toHaveFocus()
	);
	await waitFor(() => expect(view.getByRole('status')).toHaveTextContent('“UNESCO” removed.'));
});

it('says the list is empty once the last term is removed, and returns to the field', async () => {
	const view = render(GlossaryList, { locked: false });
	for (const term of ['UNESCO', 'Ki-Zerbo'])
		await fireEvent.click(view.getByRole('button', { name: `Remove “${term}”` }));
	expect(get(options).glossary).toEqual([]);
	expect(view.queryByRole('list')).toBeNull();
	expect(view.getByText('No names or terms yet.')).toBeInTheDocument();
	await waitFor(() => expect(view.getByLabelText('Add a name or term')).toHaveFocus());
});

it('explains a rejected entry beside the field, and clears it on the next edit', async () => {
	const view = render(GlossaryList, { locked: false });
	const field = view.getByLabelText('Add a name or term');
	const add = view.getByRole('button', { name: 'Add' });

	await fireEvent.input(field, { target: { value: '   ' } });
	expect(add).toBeDisabled();

	await fireEvent.input(field, { target: { value: ' unesco ' } });
	await fireEvent.click(add);
	const alert = view.getByRole('alert');
	expect(alert).toHaveTextContent('“unesco” is already in the list.');
	expect(field).toHaveAttribute('aria-invalid', 'true');
	expect(field).toHaveAttribute('aria-describedby', alert.id);

	await fireEvent.input(field, { target: { value: 'x'.repeat(201) } });
	expect(view.queryByRole('alert')).toBeNull();
	expect(field).not.toHaveAttribute('aria-invalid');
	await fireEvent.click(add);
	expect(view.getByRole('alert')).toHaveTextContent('An entry can be at most 200 characters long.');

	expect(get(options).glossary).toEqual(['UNESCO', 'Ki-Zerbo']);
});

it('refuses a term past the limit', async () => {
	const full = Array.from({ length: MAX_GLOSSARY_TERMS }, (_, i) => `Term ${i}`);
	options.set({ ...VOXTRAL, glossary: full });
	const view = render(GlossaryList, { locked: false });
	await fireEvent.input(view.getByLabelText('Add a name or term'), {
		target: { value: 'One more' }
	});
	await fireEvent.click(view.getByRole('button', { name: 'Add' }));
	expect(view.getByRole('alert')).toHaveTextContent(
		'The list holds up to 100 names and terms. Remove one first.'
	);
	expect(get(options).glossary).toEqual(full);
});

it('cannot be changed while setup is locked', async () => {
	const view = render(GlossaryList, { locked: true });
	expect(view.getByLabelText('Add a name or term')).toBeDisabled();
	expect(view.getByRole('button', { name: 'Add' })).toBeDisabled();
	const remove = view.getByRole('button', { name: 'Remove “UNESCO”' });
	expect(remove).toBeDisabled();
	await fireEvent.click(remove);
	expect(get(options).glossary).toEqual(['UNESCO', 'Ki-Zerbo']);
});

it('is translated, with each language’s own quotation marks', () => {
	setLocale('fr');
	const fr = render(GlossaryList, { locked: false });
	expect(fr.getByRole('group', { name: 'Noms et termes' })).toBeInTheDocument();
	expect(fr.getByRole('button', { name: 'Retirer « UNESCO »' })).toBeInTheDocument();
	fr.unmount();
	setLocale('de');
	const de = render(GlossaryList, { locked: false });
	expect(de.getByRole('group', { name: 'Namen und Begriffe' })).toBeInTheDocument();
	expect(de.getByRole('button', { name: '„UNESCO“ entfernen' })).toBeInTheDocument();
});
