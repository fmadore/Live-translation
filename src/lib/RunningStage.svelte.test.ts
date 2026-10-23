import { afterEach, beforeEach, expect, it } from 'vitest';
import { render } from '@testing-library/svelte';
import RunningStage from './RunningStage.svelte';
import { options, stageView, statusMessage } from './stores';
import { DEFAULT_START_OPTIONS } from './startOptions';
import { setLocale } from './i18n';

// S4: the running stage above the transcript, in each of the stage's two views.

beforeEach(() => {
	setLocale('en');
	options.set({ ...DEFAULT_START_OPTIONS });
	statusMessage.set('');
	stageView.set('live');
});
afterEach(() => {
	statusMessage.set('');
	stageView.set('live');
});

it('shows the turns on screen in the live view, and the status line in either', async () => {
	statusMessage.set('Reconnecting to the provider');
	const view = render(RunningStage, { props: { statusText: 'Reconnecting to the provider' } });
	expect(view.getByRole('heading', { name: 'On screen now' })).toBeInTheDocument();
	expect(view.getByText('Reconnecting to the provider')).toHaveAttribute('aria-hidden', 'true');

	stageView.set('transcript');
	await Promise.resolve();
	expect(view.queryByRole('heading', { name: 'On screen now' })).toBeNull();
	expect(view.getByText('Reconnecting to the provider')).toBeInTheDocument();
});
