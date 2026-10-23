import { beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import { createLaunchController, type LaunchPort } from './launchController.svelte';
import { createSessionController } from './sessionController';
import { applyStatus, describeStatus, options, statusMessage } from './stores';
import { DEFAULT_START_OPTIONS } from './startOptions';
import { en } from './i18n/en';
import { locale } from './i18n';
import type { AppError } from './errors';
import type { StartOptions, StatusUpdate } from './types';

// Start, Rehearse and the start shortcut, against the real session controller and stores with
// the core's start command scripted, as the page wires them.

const translation: StartOptions = {
	...DEFAULT_START_OPTIONS,
	mode: 'translate',
	provider: 'gemini',
	targetLanguage: 'en',
	source: 'system'
};

let ready: { engine: boolean; application: boolean; profileBusy: boolean };

function setup(
	port: Partial<LaunchPort> = {},
	startSession = vi.fn().mockResolvedValue(undefined)
) {
	const session = createSessionController({
		startSession,
		stopSession: vi.fn().mockResolvedValue(undefined),
		pauseSession: vi.fn().mockResolvedValue(undefined)
	});
	const launching = vi.fn();
	const launch = createLaunchController({
		browserMode: false,
		start: (selected) => session.start(selected),
		sessionBusy: session.busy,
		profileBusy: () => ready.profileBusy,
		engineReady: () => ready.engine,
		applicationReady: () => ready.application,
		launching,
		...port
	});
	/** A status from the core, as the page's listener hands it on. */
	const status = (update: StatusUpdate) => applyStatus(update, { starting: get(session.starting) });
	return { session, launch, launching, startSession, status };
}

beforeEach(() => {
	locale.set('en');
	applyStatus({ state: 'idle' });
	options.set(translation);
	statusMessage.set('');
	ready = { engine: true, application: true, profileBusy: false };
});

describe('what blocks a start', () => {
	it('is nothing on a ready setup, and each blocker on its own', () => {
		const { launch } = setup();
		expect(launch.canStart).toBe(true);
		expect(launch.canRehearse).toBe(true);

		ready.profileBusy = true;
		expect(launch.blockers.busy).toBe(true);
		expect([launch.canStart, launch.canRehearse]).toEqual([false, false]);
		ready.profileBusy = false;

		ready.engine = false;
		expect([launch.canStart, launch.canRehearse]).toEqual([false, false]);
		ready.engine = true;

		// A rehearsal plays the bundled sample, so a missing application holds Start alone.
		ready.application = false;
		expect([launch.canStart, launch.canRehearse]).toEqual([false, true]);
		ready.application = true;

		options.set({ ...translation, provider: 'openai', targetLanguage: 'sw' });
		expect(launch.languageError).toBe('OpenAI does not support Swahili — choose another language.');
		expect([launch.canStart, launch.canRehearse]).toEqual([false, false]);
	});

	it('is a browser preview, which has no core to start', () => {
		const { launch } = setup({ browserMode: true });
		expect(launch.blockers.busy).toBe(true);
		expect(launch.canStart).toBe(false);
	});

	it('offers no rehearsal for the built-in demonstration, which plays its own timeline', () => {
		options.set({ ...DEFAULT_START_OPTIONS, provider: 'ondevice' });
		const { launch } = setup();
		expect([launch.canStart, launch.canRehearse]).toEqual([true, false]);
	});
});

// Review U7: `start_session` answers before any source has reported. In between, the run is
// open but not yet running, and Start used to be enabled again for that moment.
it('keeps Start held from the start until the run’s first status, and frees it at the end', async () => {
	const { launch, startSession, status } = setup();
	await launch.start();
	expect(startSession).toHaveBeenCalledOnce();
	expect(launch.opening).toBe(true);
	expect(launch.blockers.busy).toBe(true);
	expect(launch.canStart).toBe(false);
	// A second press, or the shortcut, lands in that window and is refused.
	await launch.start();
	expect(startSession).toHaveBeenCalledOnce();

	status({ state: 'connecting', origin: 'system' });
	status({ state: 'running', origin: 'system' });
	expect(launch.opening).toBe(false);
	expect(launch.canStart).toBe(false);
	status({ state: 'idle' });
	expect(launch.canStart).toBe(true);
});

it('frees Start again when the start itself fails', async () => {
	const failure: AppError = { id: 'error.providerRejected', detail: '401' };
	const { launch } = setup({}, vi.fn().mockRejectedValue(failure));
	await launch.start();
	expect(get(statusMessage)).toEqual(failure);
	expect(launch.canStart).toBe(true);
});

// Review U1: every source of a run fails, and the core keeps what is left of that session
// until the next start stops it — so that stop's whole-session Idle, carrying the old failure,
// arrives while the rehearsal is starting.
it('keeps the rehearsal’s rail after a run that ended by itself, without the old failure', async () => {
	const failure: AppError = { id: 'error.whisperSession', detail: 'decoder stalled' };
	const { launch, status, startSession } = setup();
	await launch.start();
	status({ state: 'running', origin: 'system' });
	status({ state: 'error', origin: 'system', message: failure });
	expect(launch.canRehearse).toBe(true);
	expect(launch.rehearsing).toBe(false);

	startSession.mockImplementationOnce(async () => status({ state: 'idle', message: failure }));
	await launch.rehearse();
	expect(startSession).toHaveBeenLastCalledWith(expect.objectContaining({ rehearsal: 'fr' }));
	status({ state: 'connecting', origin: 'system' });
	status({ state: 'running', origin: 'system' });
	expect(launch.rehearsalRunning).toBe(true);
	expect(get(statusMessage)).toBe('');

	// The next live start is not a rehearsal, and none is shown once the run is over.
	status({ state: 'idle' });
	expect(launch.rehearsalRunning).toBe(false);
	await launch.start();
	expect(startSession).toHaveBeenLastCalledWith(
		expect.not.objectContaining({ rehearsal: expect.anything() })
	);
	status({ state: 'running', origin: 'system' });
	expect(launch.rehearsalRunning).toBe(false);
});

describe('a refused launch', () => {
	it('says why on the status line when the operator can act on it, worded where it is read', async () => {
		const { launch, launching, startSession } = setup();
		ready.application = false;
		await launch.start();
		expect(describeStatus(get(statusMessage), en, 'en')).toBe(en.applications.missing);

		options.set({ ...translation, provider: 'openai', targetLanguage: 'sw' });
		await launch.rehearse();
		expect(describeStatus(get(statusMessage), en, 'en')).toBe(
			'OpenAI does not support Swahili — choose another language.'
		);
		expect(startSession).not.toHaveBeenCalled();
		expect(launching).not.toHaveBeenCalled();
	});

	it('stays quiet when the button says it already, or a start is under way', async () => {
		const { launch } = setup();
		ready.engine = false;
		await launch.start();
		ready.engine = true;
		ready.profileBusy = true;
		await launch.start();
		expect(get(statusMessage)).toBe('');
	});

	it('clears the last capture failure only for a launch that goes ahead', async () => {
		const { launch, launching } = setup();
		await launch.start();
		expect(launching).toHaveBeenCalledOnce();
	});
});

it('rehearses on the language the room is not reading, and in English for the subtitle engines', () => {
	const { launch } = setup();
	expect(launch.fixtureLanguage).toBe('fr');
	options.set({ ...translation, targetLanguage: 'fr' });
	expect(launch.fixtureLanguage).toBe('en');
	options.set({ ...DEFAULT_START_OPTIONS, provider: 'mistral', targetLanguage: 'fr' });
	expect(launch.fixtureLanguage).toBe('en');
	options.set({ ...DEFAULT_START_OPTIONS, provider: 'ondevice', targetLanguage: 'fr' });
	expect(launch.fixtureLanguage).toBe('fr');
});
