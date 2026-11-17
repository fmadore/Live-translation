import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render } from '@testing-library/svelte';
import { get } from 'svelte/store';
import LanguageStep from './LanguageStep.svelte';
import { createSetupActions } from '../setupActions';
import { options } from '../stores';
import { DEFAULT_START_OPTIONS } from '../startOptions';
import { setLocale } from '../i18n';
import type { StartOptions } from '../types';

function mount(setup: Partial<StartOptions>, { locked = false, languageError = '' } = {}) {
	options.set({ ...DEFAULT_START_OPTIONS, ...setup });
	const actions = createSetupActions({
		locked: () => locked,
		invalidateAudioTest: vi.fn(),
		refreshDevices: vi.fn()
	});
	return render(LanguageStep, { props: { actions, locked, languageError } });
}

beforeEach(() => setLocale('en'));

describe('Whisper translation', () => {
	const whisperTranslate: Partial<StartOptions> = {
		mode: 'translate',
		provider: 'whisper-translate',
		targetLanguage: 'en',
		spokenLanguage: null
	};

	it('captions in English, says so, and asks for the spoken language it translates from', async () => {
		const view = mount(whisperTranslate);
		expect(view.getByRole('heading', { name: 'The room reads' })).toBeInTheDocument();
		expect(view.getByRole('combobox', { name: 'Caption language' })).toHaveValue('EN · English');
		expect(view.getByText(/translates into English only/)).toBeInTheDocument();
		const spoken = view.getByRole('combobox', { name: 'Spoken language' });
		expect(spoken).toHaveValue('');
		await fireEvent.change(spoken, { target: { value: 'fr' } });
		expect(get(options).spokenLanguage).toBe('fr');
		await fireEvent.change(spoken, { target: { value: '' } });
		expect(get(options).spokenLanguage).toBeNull();
	});

	it('says why a caption language other than English cannot start', () => {
		const view = mount(
			{ ...whisperTranslate, targetLanguage: 'fr' },
			{ languageError: 'Whisper translates into English only — choose English.' }
		);
		const picker = view.getByRole('combobox', { name: 'Caption language' });
		expect(picker).toHaveAttribute('aria-invalid', 'true');
		expect(picker).toHaveAccessibleDescription(/choose English/);
	});

	it('locks the spoken language with the rest of the setup', () => {
		const view = mount(whisperTranslate, { locked: true });
		expect(view.getByRole('combobox', { name: 'Spoken language' })).toBeDisabled();
	});
});

describe('the demo language cards', () => {
	const demo: Partial<StartOptions> = {
		mode: 'transcribe',
		provider: 'ondevice',
		targetLanguage: 'en'
	};

	it('offers the two scripts as pressable cards and switches between them', async () => {
		const view = mount(demo);
		expect(view.getByRole('heading', { name: 'Demo language' })).toBeInTheDocument();
		const english = view.getByRole('button', { name: /^EN/ });
		const french = view.getByRole('button', { name: /^FR/ });
		expect(english).toHaveAttribute('aria-pressed', 'true');
		expect(french).toHaveAttribute('aria-pressed', 'false');
		await fireEvent.click(french);
		expect(get(options).targetLanguage).toBe('fr');
		expect(french).toHaveAttribute('aria-pressed', 'true');
		expect(english).toHaveAttribute('aria-pressed', 'false');
		// No searchable picker: the demo has two scripts and nothing else.
		expect(view.queryByRole('combobox')).toBeNull();
	});

	it('names the cards in the interface language', () => {
		setLocale('fr');
		const view = mount(demo);
		expect(view.getByRole('button', { name: /^FR/ })).toHaveTextContent('français');
	});

	it('disables both cards while setup is locked', () => {
		const view = mount(demo, { locked: true });
		for (const card of view.getAllByRole('button', { name: /^(EN|FR)/ }))
			expect(card).toBeDisabled();
	});

	it('says a language error in a status above the cards', () => {
		const view = mount(demo, { languageError: 'The demo has no script in that language.' });
		expect(view.getByRole('status')).toHaveTextContent('The demo has no script in that language.');
	});
});
