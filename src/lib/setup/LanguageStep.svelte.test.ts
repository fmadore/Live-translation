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

// Issue #249: Gemini Transcribe can be told the spoken language, from the part of Whisper's
// list that Google's table names.
describe('Gemini Transcribe', () => {
	const gemini: Partial<StartOptions> = {
		mode: 'transcribe',
		provider: 'gemini-transcribe',
		spokenLanguage: null
	};

	function choices(select: HTMLElement): string[] {
		return [...select.querySelectorAll('option')].map((o) => o.value);
	}

	it('offers automatic detection first, then only the languages Gemini knows', async () => {
		const view = mount(gemini);
		expect(view.getByRole('heading', { name: 'Spoken language' })).toBeInTheDocument();
		expect(view.getByText(/to steady short sentences/)).toBeInTheDocument();
		expect(view.queryByText(/auto-detects the spoken language/)).toBeNull();
		const spoken = view.getByRole('combobox', { name: 'Spoken language' });
		expect(spoken).toHaveValue('');
		const offered = choices(spoken);
		expect(offered[0]).toBe('');
		expect(offered).toHaveLength(72);
		expect(offered).toEqual(expect.arrayContaining(['fr', 'en', 'zh', 'jw', 'tl']));
		// Whisper hears Yoruba and Welsh; Google's table has no row for either.
		expect(offered).not.toContain('yo');
		expect(offered).not.toContain('cy');
		await fireEvent.change(spoken, { target: { value: 'fr' } });
		expect(get(options).spokenLanguage).toBe('fr');
		await fireEvent.change(spoken, { target: { value: '' } });
		expect(get(options).spokenLanguage).toBeNull();
	});

	it('reads a Whisper language it cannot name as automatic detection, and keeps it for Whisper', () => {
		const view = mount({ ...gemini, spokenLanguage: 'yo' });
		expect(view.getByRole('combobox', { name: 'Spoken language' })).toHaveValue('');
		expect(get(options).spokenLanguage).toBe('yo');
	});

	it('shows a shared choice it knows, and locks it with the rest of the setup', () => {
		const view = mount({ ...gemini, spokenLanguage: 'de' }, { locked: true });
		const spoken = view.getByRole('combobox', { name: 'Spoken language' });
		expect(spoken).toHaveValue('de');
		expect(spoken).toBeDisabled();
	});

	it('words its hint in each interface language', () => {
		for (const [locale, hint] of [
			['fr', /fiabiliser les phrases courtes/],
			['de', /Kurze Sätze werden dann sicherer erkannt/]
		] as const) {
			setLocale(locale);
			const view = mount(gemini);
			expect(view.getByText(hint)).toBeInTheDocument();
			view.unmount();
		}
	});

	it('leaves Voxtral with its auto-detect note and no picker', () => {
		const view = mount({ ...gemini, provider: 'mistral' });
		expect(view.getByText(/Voxtral auto-detects the spoken language/)).toBeInTheDocument();
		expect(view.queryByRole('combobox', { name: 'Spoken language' })).toBeNull();
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
