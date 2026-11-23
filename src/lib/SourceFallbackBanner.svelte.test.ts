import { beforeEach, expect, it, vi } from 'vitest';
import { fireEvent, render } from '@testing-library/svelte';
import SourceFallbackBanner from './SourceFallbackBanner.svelte';
import { setLocale } from './i18n';
import type { FallbackOffer } from './sourceFallback.svelte';
import type { Origin } from './types';

beforeEach(() => setLocale('en'));

function banner(
	offer: FallbackOffer,
	{
		origins = ['system'] as Origin[],
		switching = false,
		reconnecting = (_: Origin): boolean => false
	} = {}
) {
	const onContinue = vi.fn();
	const onDismiss = vi.fn();
	const view = render(SourceFallbackBanner, {
		props: {
			offer,
			origins,
			from: () => 'gemini',
			reconnecting,
			switching,
			onContinue,
			onDismiss
		}
	});
	return { view, onContinue, onDismiss };
}

const english: FallbackOffer = {
	to: 'whisper-translate',
	model: 'small',
	gpu: true,
	note: { kind: 'keepsEnglish' }
};

it('names its region and says which source stopped, on which engine', () => {
	const { view } = banner(english);
	expect(view.getByRole('region', { name: 'Continue the session' })).toBeInTheDocument();
	expect(view.getByText('System audio: Google Gemini stopped captioning.')).toBeInTheDocument();
	view.unmount();
});

it.each([
	[english, 'Continue with offline Whisper', 'Offline Whisper will keep translating into English.'],
	[
		{ ...english, to: 'whisper', model: 'turbo', note: { kind: 'turboCannotTranslate' } },
		'Continue in the spoken language',
		'Large v3 Turbo cannot translate: captions will continue in the spoken language.'
	],
	[
		{ ...english, to: 'whisper', note: { kind: 'cannotTranslate', language: 'fr' } },
		'Continue in the spoken language',
		'Offline Whisper cannot translate into French: captions will continue in the spoken language.'
	]
] as [FallbackOffer, string, string][])(
	'says on the button what the room will read (%#)',
	(offer, name, description) => {
		const { view } = banner(offer);
		expect(view.getByRole('button', { name })).toHaveAccessibleDescription(description);
		view.unmount();
	}
);

it('has no line under the button for a transcription', () => {
	const { view } = banner({ ...english, to: 'whisper', note: null });
	expect(view.getByRole('button', { name: 'Continue with offline Whisper' })).not.toHaveAttribute(
		'aria-describedby'
	);
	view.unmount();
});

it('says both sources continue, and that one is still reconnecting', () => {
	const { view } = banner(english, {
		origins: ['microphone', 'system'],
		reconnecting: (origin) => origin === 'microphone'
	});
	expect(
		view.getByRole('button', { name: 'Continue with offline Whisper' })
	).toHaveAccessibleDescription(
		'Offline Whisper will keep translating into English. Both sources continue with it.'
	);
	expect(
		view.getByText(
			'Room mic: Google Gemini has been reconnecting for 30 seconds. It keeps trying until you choose.'
		)
	).toBeInTheDocument();
	view.unmount();
});

it('cannot be pressed twice while it switches', () => {
	const { view } = banner(english, { switching: true });
	expect(view.getByRole('button', { name: 'Switching engine…' })).toBeDisabled();
	expect(view.getByRole('button', { name: 'Not now' })).toBeDisabled();
	view.unmount();
});

it('continues, or ends the held run with Not now', async () => {
	const { view, onContinue, onDismiss } = banner(english);
	await fireEvent.click(view.getByRole('button', { name: 'Continue with offline Whisper' }));
	expect(onContinue).toHaveBeenCalledOnce();
	await fireEvent.click(view.getByRole('button', { name: 'Not now' }));
	expect(onDismiss).toHaveBeenCalledOnce();
	view.unmount();
});
