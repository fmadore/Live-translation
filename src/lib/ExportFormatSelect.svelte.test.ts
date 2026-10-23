import { afterEach, beforeEach, expect, it } from 'vitest';
import { fireEvent, render, within } from '@testing-library/svelte';
import { get } from 'svelte/store';
import ExportFormatSelect from './ExportFormatSelect.svelte';
import { exportFormat } from './exportFormat';
import { setLocale } from './i18n';

// S6: the transcript toolbar and the history view choose the format with one select, over the
// one remembered preference.

beforeEach(() => {
	setLocale('en');
	exportFormat.set('markdown');
});
afterEach(() => exportFormat.set('markdown'));

it('offers the four formats and keeps the choice in the shared preference', async () => {
	const toolbar = render(ExportFormatSelect, { props: { compact: true } });
	const history = render(ExportFormatSelect);
	const [small, field] = [toolbar, history].map((view) =>
		within(view.container).getByRole('combobox', { name: 'Export format' })
	);
	expect([...small.querySelectorAll('option')].map((o) => o.textContent)).toEqual([
		'Markdown (.md)',
		'Plain text (.txt)',
		'WebVTT (.vtt)',
		'SubRip (.srt)'
	]);
	// The history view's is the form field, with its chevron; the toolbar's is the small one.
	expect(field.closest('.ui-select')).not.toBeNull();
	expect(small.closest('.ui-select')).toBeNull();

	await fireEvent.change(small, { target: { value: 'srt' } });
	expect(get(exportFormat)).toBe('srt');
	expect(field).toHaveValue('srt');
});

it('words plain text in the interface language and can be held while saving', async () => {
	setLocale('fr');
	const view = render(ExportFormatSelect, { props: { disabled: true } });
	const select = view.getByRole('combobox', { name: 'Format d’export' });
	expect(select).toBeDisabled();
	expect(select).toHaveTextContent('Texte brut (.txt)');
});
