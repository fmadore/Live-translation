// The Captions in Zoom dialog against Tauri's own IPC mocks, so the real `tauri.ts` carries
// every command under its real name and arguments, and the test plays the core.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, waitFor } from '@testing-library/svelte';
import ZoomCaptionsDialog from './ZoomCaptionsDialog.svelte';
import { createZoomCaptions, ZOOM_GUIDE_URL, type ZoomCaptions } from './zoomCaptions.svelte';
import { mockTauri, type TauriMock } from './testing/tauriMock';
import { DEFAULT_START_OPTIONS } from './startOptions';
import { setLocale } from './i18n';
import { options } from './stores';
import type { StartOptions, ZoomStatus } from './types';

/** A link of the shape Zoom hands out; the core, not this test, decides what is valid. */
const LINK =
	'https://wmcc.zoom.us/closedcaption?id=200610693&ns=GZHkEA==&expire=86400&sparams=id%2Cns%2Cexpire&signature=nYtXJqRKCW';

/** Translating both sources into English and French: every choice the dialog offers. */
const TWO_OF_EACH: StartOptions = {
	...DEFAULT_START_OPTIONS,
	mode: 'translate',
	provider: 'gemini',
	source: 'both',
	targetLanguage: 'en',
	secondTargetLanguage: 'fr'
};

const status = (patch: Partial<ZoomStatus> = {}): ZoomStatus => ({
	state: 'sharing',
	lane: 0,
	origins: ['microphone', 'system'],
	sent: 0,
	dropped: 0,
	...patch
});

let core: TauriMock;

beforeEach(() => {
	setLocale('en');
	options.set({ ...TWO_OF_EACH });
	core = mockTauri();
	// As `zoom.rs` answers a start that Zoom accepted: sharing what it was asked to share.
	core.handle('start_zoom_captions', ({ lane, origins }) =>
		status({ lane: lane as ZoomStatus['lane'], origins: origins as ZoomStatus['origins'] })
	);
});

afterEach(async () => {
	cleanup();
	await core.dispose();
	vi.restoreAllMocks();
	setLocale('en');
});

function mount(zoom: ZoomCaptions = createZoomCaptions()) {
	const onClose = vi.fn();
	const view = render(ZoomCaptionsDialog, { props: { zoom, onClose } });
	const link = view.getByLabelText('Caption link (API token)') as HTMLInputElement;
	const start = () => view.getByRole('button', { name: 'Start sharing' });
	return { ...view, zoom, onClose, link, start };
}

describe('starting to share', () => {
	it('is a labelled modal dialog that Escape closes', async () => {
		const { getByRole, onClose } = mount();
		const dialog = getByRole('dialog');
		expect(dialog).toHaveAttribute('aria-modal', 'true');
		expect(dialog).toHaveAccessibleName('Captions in Zoom');
		await fireEvent.keyDown(window, { key: 'Escape' });
		expect(onClose).toHaveBeenCalledOnce();
	});

	it('sends the link, the caption language and the sources chosen', async () => {
		const { getByRole, link, start } = mount();
		// Pasted from the meeting chat, with the line break that came along.
		await fireEvent.input(link, { target: { value: `  ${LINK}\n` } });
		await fireEvent.change(getByRole('combobox', { name: 'Caption language' }), {
			target: { value: '1' }
		});
		await fireEvent.click(getByRole('checkbox', { name: 'Room (microphone)' }));
		await fireEvent.click(start());

		await waitFor(() => expect(core.callsTo('start_zoom_captions')).toHaveLength(1));
		expect(core.callsTo('start_zoom_captions')[0]).toEqual({
			url: LINK,
			lane: 1,
			origins: ['system']
		});
	});

	it('offers the languages by name, and both sources ticked to begin with', () => {
		const { getByRole } = mount();
		const language = getByRole('combobox', { name: 'Caption language' });
		expect(language).toHaveValue('0');
		expect([...language.querySelectorAll('option')].map((o) => o.textContent)).toEqual([
			'English',
			'French'
		]);
		expect(getByRole('checkbox', { name: 'Room (microphone)' })).toBeChecked();
		expect(getByRole('checkbox', { name: 'Remote (system audio)' })).toBeChecked();
	});

	it('asks for no choice the setup does not offer, and shares its only one', async () => {
		options.set({ ...DEFAULT_START_OPTIONS, source: 'system' });
		const { queryByRole, link, start } = mount();
		expect(queryByRole('combobox')).toBeNull();
		expect(queryByRole('checkbox')).toBeNull();

		await fireEvent.input(link, { target: { value: LINK } });
		await fireEvent.click(start());
		await waitFor(() => expect(core.callsTo('start_zoom_captions')).toHaveLength(1));
		expect(core.callsTo('start_zoom_captions')[0]).toEqual({
			url: LINK,
			lane: 0,
			origins: ['system']
		});
	});

	it('cannot start without a link, or with no source to share', async () => {
		const { getByRole, link, start } = mount();
		expect(start()).toBeDisabled();
		await fireEvent.input(link, { target: { value: '   ' } });
		expect(start()).toBeDisabled();

		await fireEvent.input(link, { target: { value: LINK } });
		expect(start()).toBeEnabled();
		await fireEvent.click(getByRole('checkbox', { name: 'Room (microphone)' }));
		await fireEvent.click(getByRole('checkbox', { name: 'Remote (system audio)' }));
		expect(start()).toBeDisabled();
	});

	// The link lets anyone post captions to the meeting, and the operator's screen may be on a
	// projector: masked, and kept nowhere that outlives the app.
	it('treats the link as a password and keeps it out of storage and the console', async () => {
		const consoleCalls = (['log', 'info', 'warn', 'error', 'debug'] as const).map((method) =>
			vi.spyOn(console, method).mockImplementation(() => {})
		);
		const stored = vi.spyOn(Storage.prototype, 'setItem');
		const { link, start, getByRole } = mount();
		expect(link).toHaveAttribute('type', 'password');
		expect(link).toHaveAttribute('autocomplete', 'off');
		expect(link).toHaveAttribute('spellcheck', 'false');
		expect(link).toHaveAccessibleDescription(/Treated like a password/);

		await fireEvent.input(link, { target: { value: LINK } });
		await fireEvent.click(start());
		await waitFor(() => expect(getByRole('button', { name: 'Stop sharing' })).toBeInTheDocument());

		expect(stored).not.toHaveBeenCalled();
		for (const storage of [localStorage, sessionStorage]) {
			for (let i = 0; i < storage.length; i++) {
				expect(storage.getItem(storage.key(i)!)).not.toContain('closedcaption');
			}
		}
		for (const spy of consoleCalls) {
			for (const args of spy.mock.calls) expect(JSON.stringify(args)).not.toContain(LINK);
		}
	});

	it('words a refused link, with the detail the core gave', async () => {
		core.handle('start_zoom_captions', () => {
			throw { id: 'error.zoomRefused', detail: 'HTTP 403' };
		});
		const { link, start, findByRole, getByRole } = mount();
		await fireEvent.input(link, { target: { value: LINK } });
		await fireEvent.click(start());

		expect(await findByRole('alert')).toHaveTextContent(
			/^Zoom refused the caption link: .* Copy a new API token \(HTTP 403\)$/
		);
		expect(getByRole('status')).toHaveTextContent('Not sharing.');
		// Still there to correct and try again.
		expect(link).toBeEnabled();
		expect(start()).toBeEnabled();
	});
});

describe('while sharing', () => {
	async function sharing() {
		const view = mount();
		await fireEvent.input(view.link, { target: { value: LINK } });
		view.start().focus();
		await fireEvent.click(view.start());
		await view.findByRole('button', { name: 'Stop sharing' });
		return view;
	}

	it('locks what is shared until sharing stops', async () => {
		const { getByRole, queryByRole, link } = await sharing();
		expect(link).toBeDisabled();
		expect(getByRole('combobox', { name: 'Caption language' })).toBeDisabled();
		expect(getByRole('checkbox', { name: 'Room (microphone)' })).toBeDisabled();
		expect(getByRole('checkbox', { name: 'Remote (system audio)' })).toBeDisabled();
		expect(queryByRole('button', { name: 'Start sharing' })).toBeNull();
	});

	// Start goes as sharing begins; the keyboard goes to the next thing to do.
	it('hands the keyboard to the test caption once sharing starts', async () => {
		const { getByRole } = await sharing();
		await waitFor(() => expect(getByRole('button', { name: 'Send test caption' })).toHaveFocus());
	});

	it('sends the test caption in the interface language', async () => {
		const { getByRole, findByRole } = await sharing();
		await fireEvent.click(getByRole('button', { name: 'Send test caption' }));
		await waitFor(() => expect(core.callsTo('send_zoom_test_caption')).toHaveLength(1));
		expect(core.callsTo('send_zoom_test_caption')[0]).toEqual({
			text: 'Test caption from Live Translation.'
		});

		setLocale('fr');
		await fireEvent.click(await findByRole('button', { name: 'Envoyer un sous-titre de test' }));
		await waitFor(() => expect(core.callsTo('send_zoom_test_caption')).toHaveLength(2));
		expect(core.callsTo('send_zoom_test_caption')[1]).toEqual({
			text: 'Sous-titre de test envoyé par Live Translation.'
		});
	});

	it('stops, keeps the link for the next start, and hands the keyboard back to Start', async () => {
		const { getByRole, link, start, findByRole } = await sharing();
		const stop = getByRole('button', { name: 'Stop sharing' });
		stop.focus();
		await fireEvent.click(stop);

		await findByRole('button', { name: 'Start sharing' });
		expect(core.callsTo('stop_zoom_captions')).toHaveLength(1);
		expect(getByRole('status')).toHaveTextContent('Not sharing.');
		expect(link).toBeEnabled();
		expect(link).toHaveValue(LINK);
		await waitFor(() => expect(start()).toHaveFocus());
	});
});

describe('the status line', () => {
	it.each([
		['off', 'Not sharing.'],
		['sharing', 'Sharing with Zoom.'],
		['waiting', 'Zoom says the meeting has not started yet. Captions will appear once it has.'],
		['refused', 'Zoom refused this link. It may have expired: copy a new API token.'],
		['unreachable', 'Cannot reach Zoom. Trying again with each new caption.']
	] as const)('says what %s means', (state, sentence) => {
		const zoom = createZoomCaptions();
		zoom.apply(status({ state }));
		expect(mount(zoom).getByRole('status')).toHaveTextContent(sentence);
	});

	// A live region speaks when the state changes, never as a number climbs: the counts are
	// written beside it, outside it.
	it('counts what was sent and skipped outside the live region', async () => {
		const zoom = createZoomCaptions();
		zoom.apply(status({ sent: 1 }));
		const { getByRole, getByText } = mount(zoom);
		expect(getByText('1 caption sent.')).toBeInTheDocument();

		zoom.apply(status({ state: 'unreachable', sent: 12, dropped: 3 }));
		await waitFor(() =>
			expect(
				getByText(
					'12 captions sent. 3 captions were skipped because they could not be delivered in time.'
				)
			).toBeInTheDocument()
		);
		expect(getByRole('status')).toHaveTextContent(
			/^Cannot reach Zoom. Trying again with each new caption.$/
		);
	});

	it('shows what is shared, not what the setup would share now', () => {
		options.set({ ...DEFAULT_START_OPTIONS, source: 'microphone' });
		const zoom = createZoomCaptions();
		zoom.apply(status({ origins: ['system'] }));
		const { getByRole } = mount(zoom);
		expect(getByRole('checkbox', { name: 'Room (microphone)' })).not.toBeChecked();
		expect(getByRole('checkbox', { name: 'Remote (system audio)' })).toBeChecked();
	});
});

describe('the setup guide', () => {
	it('is a link to the guide that says it opens in the browser', () => {
		const link = mount().getByRole('link', { name: /^Step-by-step setup guide/ });
		expect(link).toHaveAttribute('href', ZOOM_GUIDE_URL);
		expect(link).toHaveAccessibleName(/\(opens in your browser\)$/);
		expect(ZOOM_GUIDE_URL).toBe(
			'https://github.com/fmadore/Live-translation/blob/main/docs/zoom-captions.md'
		);
	});

	// WebView2 would open the href in a popup of its own, so in the app the core opens it.
	it('asks the core to open it in the default browser', async () => {
		const open = vi.spyOn(window, 'open');
		await fireEvent.click(mount().getByRole('link', { name: /^Step-by-step setup guide/ }));
		await waitFor(() => expect(core.callsTo('open_zoom_guide')).toHaveLength(1));
		expect(open).not.toHaveBeenCalled();
	});

	it('says so when Windows will not open it', async () => {
		core.handle('open_zoom_guide', () => {
			throw { id: 'error.openLink', detail: 'ShellExecute returned 2' };
		});
		const view = mount();
		await fireEvent.click(view.getByRole('link', { name: /^Step-by-step setup guide/ }));
		expect(await view.findByRole('alert')).toHaveTextContent(
			'Windows could not open the page in your browser'
		);
	});
});

describe('the controller', () => {
	// The window reloads while a share runs on: the core says so when asked.
	it('resyncs with the core', async () => {
		core.handle('zoom_captions_status', () => status({ sent: 4 }));
		const zoom = createZoomCaptions();
		await zoom.resync();
		expect(zoom.active).toBe(true);
		expect(zoom.status.sent).toBe(4);
	});

	it('keeps an event that overtook the resync answer', async () => {
		let answer: (s: ZoomStatus) => void = () => {};
		core.handle('zoom_captions_status', () => new Promise<ZoomStatus>((r) => (answer = r)));
		const zoom = createZoomCaptions();
		const asked = zoom.resync();
		await waitFor(() => expect(core.callsTo('zoom_captions_status')).toHaveLength(1));
		zoom.apply(status({ state: 'refused', httpStatus: 403 }));
		answer(status({ sent: 1 }));
		await asked;
		expect(zoom.status.state).toBe('refused');
	});
});
