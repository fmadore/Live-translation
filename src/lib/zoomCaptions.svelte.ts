// Captions in Zoom (`src-tauri/src/zoom.rs`, docs/zoom-captions.md): where sharing stands, the
// caption link the operator pasted, the language and sources chosen for it, and the three
// things the operator can do about it.
//
// The link is a credential for the meeting's captions, so it is held here, in memory, and
// nowhere else: not in a persisted store, the saved setup, a meeting profile, history or a log.
// A reload of the window forgets it; the core keeps sharing regardless, and `resync` says so.

import { asStatus, type AppError } from './errors';
import { api, isTauri } from './tauri';
import { ORIGINS, type Lane, type Origin, type ZoomStatus } from './types';

/** The step-by-step setup guide, for the host's settings in Zoom as much as for this app. The
 *  core opens this exact page and no other (`GUIDE_URL` in `src-tauri/src/zoom.rs`, whose test
 *  keeps the two in step). */
export const ZOOM_GUIDE_URL =
	'https://github.com/fmadore/Live-translation/blob/main/docs/zoom-captions.md';

/** Open the guide in the default browser, as `openSupportPage` opens its page: through the
 *  core in the app, since WebView2 would open a `target="_blank"` link in a popup of its own,
 *  and in a new tab in a browser preview. */
export async function openZoomGuide(): Promise<void> {
	if (isTauri()) {
		await api.openZoomGuide();
	} else {
		window.open(ZOOM_GUIDE_URL, '_blank', 'noopener,noreferrer');
	}
}

/** What the window assumes before the core has answered, as `ZoomStatus::off` says it: nothing
 *  is shared until the operator starts it. */
const OFF: ZoomStatus = { state: 'off', lane: 0, origins: [], sent: 0, dropped: 0 };

type ZoomPort = Pick<
	typeof api,
	'startZoomCaptions' | 'stopZoomCaptions' | 'sendZoomTestCaption' | 'zoomCaptionsStatus'
>;

export type ZoomCaptions = ReturnType<typeof createZoomCaptions>;

export function createZoomCaptions(port: ZoomPort = api) {
	// Raw: each status replaces the last whole, after every caption while sharing, so there is
	// nothing to gain from proxying its fields one by one. The same goes for the origins.
	let status = $state.raw<ZoomStatus>(OFF);
	let link = $state('');
	let lane = $state<Lane>(0);
	let origins = $state.raw<Origin[]>([...ORIGINS]);
	let pending = $state<'start' | 'stop' | 'test' | null>(null);
	let error = $state<AppError | string | null>(null);
	// Counts the statuses the core has pushed, so a resync answer overtaken on its way back by
	// an event cannot put the older of the two on screen.
	let heard = 0;

	/** A `zoom-captions` event, or the core's answer to a start. */
	function apply(next: ZoomStatus) {
		heard++;
		status = next;
	}

	/** Ask where sharing stands: the window has just loaded, and may have reloaded while a
	 *  share it no longer remembers kept running. */
	async function resync() {
		const before = heard;
		try {
			const current = await port.zoomCaptionsStatus();
			if (current && heard === before) status = current;
		} catch {
			// Nothing the operator can act on: the window keeps saying "not sharing" until the
			// core's next event, which every delivered or dropped caption sends.
		}
	}

	/** One action at a time; a failure is kept, structured, for the dialog to word. */
	async function run(action: 'start' | 'stop' | 'test', work: () => Promise<void>) {
		if (pending) return;
		pending = action;
		error = null;
		try {
			await work();
		} catch (failure) {
			error = asStatus(failure);
		} finally {
			pending = null;
		}
	}

	return {
		get status() {
			return status;
		},
		/** Sharing is on, whether or not Zoom is taking the captions right now. */
		get active() {
			return status.state !== 'off';
		},
		get link() {
			return link;
		},
		set link(value: string) {
			link = value;
		},
		/** The caption language chosen for the next start, when the setup has two. */
		get lane() {
			return lane;
		},
		set lane(value: Lane) {
			lane = value;
		},
		/** The sources chosen for the next start, when the setup captures both. */
		get origins() {
			return origins;
		},
		/** Which action is under way, if any. */
		get pending() {
			return pending;
		},
		/** The last action's failure, until the next action. */
		get error() {
			return error;
		},
		setOrigin(origin: Origin, shared: boolean) {
			origins = ORIGINS.filter((o) => (o === origin ? shared : origins.includes(o)));
		},
		apply,
		resync,
		/** Start sharing `lane` from `sources`, with the link as pasted less the whitespace a
		 *  copy from a chat message tends to bring along. */
		start: (shareLane: Lane, sources: Origin[]) =>
			run('start', async () => {
				apply(await port.startZoomCaptions(link.trim(), shareLane, sources));
			}),
		stop: () =>
			run('stop', async () => {
				await port.stopZoomCaptions();
				// The core says so too, but the dialog should not wait on an event to show it.
				status = OFF;
			}),
		/** Send `text`, in the interface language, to everyone in the meeting. */
		sendTest: (text: string) => run('test', () => port.sendZoomTestCaption(text))
	};
}
