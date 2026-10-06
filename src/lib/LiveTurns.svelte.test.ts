import { afterEach, beforeEach, expect, it } from 'vitest';
import { render } from '@testing-library/svelte';
import LiveTurns from './LiveTurns.svelte';
import { setLocale } from './i18n';
import { currentCaptions, options } from './stores';
import { CAPTION_TAIL_CHARS } from './captionLayout';
import { DEFAULT_START_OPTIONS } from './types';

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
