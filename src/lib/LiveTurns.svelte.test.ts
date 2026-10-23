import { afterEach, beforeEach, expect, it } from 'vitest';
import { render } from '@testing-library/svelte';
import LiveTurns from './LiveTurns.svelte';
import { setLocale } from './i18n';
import { currentCaptions, options } from './stores';
import { CAPTION_TAIL_CHARS } from './captionLayout';
import { DEFAULT_START_OPTIONS } from './startOptions';

beforeEach(() => {
	setLocale('en');
	options.set({
		...DEFAULT_START_OPTIONS,
		mode: 'translate',
		provider: 'gemini',
		targetLanguage: 'en'
	});
});
afterEach(() => {
	currentCaptions.set({});
	options.set({ ...DEFAULT_START_OPTIONS });
});

// Review E7: a turn that streamed for minutes was laid out whole, at display size, on every
// interim. The stage now shows its newest part, as the overlay does.
it('shows the newest part of a long turn and of its original speech', () => {
	const words = (stem: string) => Array.from({ length: 4000 }, (_, i) => `${stem}${i}`).join(' ');
	currentCaptions.set({
		system: {
			origin: 'system',
			turnId: 1,
			text: words('word'),
			sourceText: words('mot'),
			final: false,
			startMs: 0,
			endMs: 0
		}
	});
	const view = render(LiveTurns);
	const caption = view.container.querySelector('.turn-caption')!.textContent!.trim();
	const source = view.container.querySelector('.turn-source')!.textContent!.trim();
	for (const [text, last] of [
		[caption, 'word3999'],
		[source, 'mot3999']
	]) {
		expect(text.length).toBeLessThanOrEqual(CAPTION_TAIL_CHARS + 2);
		expect(text.startsWith('… ')).toBe(true);
		expect(text.endsWith(last)).toBe(true);
	}
	view.unmount();
});

// Whisper's translate task has English and no original speech, and a second language saved for
// a cloud engine does not make its single language look like one of two.
it('shows a Whisper translation as the caption alone', () => {
	options.set({
		...DEFAULT_START_OPTIONS,
		mode: 'translate',
		provider: 'whisper-translate',
		targetLanguage: 'en',
		secondTargetLanguage: 'fr'
	});
	currentCaptions.set({
		microphone: {
			origin: 'microphone',
			turnId: 1,
			text: 'Good morning.',
			sourceText: '',
			final: true,
			startMs: 0,
			endMs: 0
		}
	});
	const view = render(LiveTurns);
	expect(view.container.querySelector('.turn-source')).toBeNull();
	expect(view.container.querySelector('.turn-caption')).toHaveTextContent('Good morning.');
	expect(view.container.querySelector('.origin-sub')).not.toHaveTextContent('English');
	view.unmount();
});

it('shows a short turn as it is', () => {
	currentCaptions.set({
		microphone: {
			origin: 'microphone',
			turnId: 1,
			text: 'Good morning.',
			sourceText: 'Bonjour.',
			final: true,
			startMs: 0,
			endMs: 0
		}
	});
	const view = render(LiveTurns);
	expect(view.getByText('Good morning.')).toBeInTheDocument();
	expect(view.getByText('Bonjour.')).toBeInTheDocument();
	view.unmount();
});

// Review U12: the operator window's captions carry their language, as the overlay's do, so
// Narrator reads them with that language's voice and Arabic lays out right to left.
it('gives each caption its language and direction, and leaves the original speech unknown', () => {
	options.set({
		...DEFAULT_START_OPTIONS,
		mode: 'translate',
		provider: 'gemini',
		targetLanguage: 'ar',
		secondTargetLanguage: 'fr'
	});
	const turn = { origin: 'system', turnId: 1, final: true, startMs: 0, endMs: 900 } as const;
	currentCaptions.set({
		system: { ...turn, text: 'مرحبا بكم', sourceText: 'Welcome' },
		'system:1': { ...turn, lane: 1, text: 'Bienvenue', sourceText: 'Welcome' }
	});
	const view = render(LiveTurns);
	expect(view.getByText('مرحبا بكم')).toHaveAttribute('lang', 'ar');
	expect(view.getByText('مرحبا بكم')).toHaveAttribute('dir', 'rtl');
	expect(view.getByText('Bienvenue')).toHaveAttribute('lang', 'fr');
	expect(view.getByText('Bienvenue')).toHaveAttribute('dir', 'ltr');
	// The original speech is shown once, on the first language, in a language not known here.
	expect(view.getByText('Welcome')).toHaveAttribute('lang', '');
	expect(view.getByText('Welcome')).toHaveAttribute('dir', 'auto');
	view.unmount();
});

it('says the language is unknown while a subtitle engine detects it', () => {
	options.set({ ...DEFAULT_START_OPTIONS, mode: 'transcribe', provider: 'mistral' });
	currentCaptions.set({
		microphone: {
			origin: 'microphone',
			turnId: 1,
			text: 'שלום',
			sourceText: '',
			final: true,
			startMs: 0,
			endMs: 0
		}
	});
	const view = render(LiveTurns);
	expect(view.getByText('שלום')).toHaveAttribute('lang', '');
	expect(view.getByText('שלום')).toHaveAttribute('dir', 'auto');
	view.unmount();
});
