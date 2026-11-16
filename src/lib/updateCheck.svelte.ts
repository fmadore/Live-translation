// The new-version notice for copies installed from GitHub (issue #217, `src-tauri/src/update.rs`):
// where the check stands, the question asked from the second launch on, and the operator's
// answers to it and to the notice.
//
// The core decides everything that matters: whether this copy is eligible, whether the switch is
// on, whether a day has passed and whether a session runs. This only asks it, a few seconds
// after launch and never while a session runs, and shows what it says. A failure of any of it is
// silent: the notice is a courtesy, and nothing about it is worth an error on screen.

import { tick } from 'svelte';
import { api } from './tauri';
import type { UpdateStatus } from './types';

type UpdatePort = Pick<
	typeof api,
	'updateStatus' | 'checkForUpdate' | 'setUpdateChecks' | 'skipUpdateVersion' | 'openReleasePage'
>;

export type UpdateCheck = ReturnType<typeof createUpdateCheck>;

/** How long after launch the check waits, so it never competes with the window's own start. */
export const CHECK_DELAY_MS = 5_000;
/** How often it looks again while a session runs: it waits for the session to end. */
export const RETRY_DELAY_MS = 60_000;

/** `settle` waits for the render that mounts the live region; a test can hold it. */
export function createUpdateCheck(port: UpdatePort = api, settle: () => Promise<void> = tick) {
	let status = $state.raw<UpdateStatus | null>(null);
	let timer: ReturnType<typeof setTimeout> | undefined;
	let disposed = false;

	/** Take the core's answer. A new copy of the start-screen line is put in its live region a
	 *  render after the region itself, so a screen reader hears it: a region mounted together
	 *  with its text is not announced. */
	async function apply(next: UpdateStatus | null | undefined) {
		if (!next || disposed) return;
		if (!status?.eligible && next.eligible) {
			status = { ...next, ask: false, available: null };
			await settle();
			if (disposed) return;
		}
		status = next;
	}

	/** Run one call to the core, keeping its answer and ignoring its failure. */
	async function run(call: () => Promise<UpdateStatus>) {
		try {
			await apply(await call());
		} catch {
			// Silent by design: offline, a store build, a core that does not know the command.
		}
	}

	/** Ask where the check stands; the first call of a launch also counts the launch. */
	function load() {
		return run(() => port.updateStatus());
	}

	/** Check a few seconds from now, and keep waiting while `running` says a session runs. The
	 *  core refuses during a session as well; waiting here keeps the notice from appearing at
	 *  the very moment a session ends. */
	function schedule(running: () => boolean, delay = CHECK_DELAY_MS) {
		clearTimeout(timer);
		// A copy that never checks has nothing to wait for.
		if (!status?.eligible || disposed) return;
		timer = setTimeout(() => {
			timer = undefined;
			if (disposed) return;
			if (running()) {
				schedule(running, RETRY_DELAY_MS);
				return;
			}
			void check();
		}, delay);
	}

	function check() {
		return run(() => port.checkForUpdate());
	}

	/** The switch, or the question's Turn on / No thanks. Turning it on checks at once: the
	 *  operator has just asked for it, and the core still keeps to once a day. */
	async function setChecks(on: boolean) {
		await run(() => port.setUpdateChecks(on));
		if (on && status?.checks === 'on') await check();
	}

	/** Not now: hide the version on offer until a newer one. */
	function notNow() {
		return run(() => port.skipUpdateVersion());
	}

	/** Open the release page in the browser. Rejects when the core could not, for the button
	 *  that asked to say so. */
	function openReleaseNotes() {
		return port.openReleasePage();
	}

	function dispose() {
		disposed = true;
		clearTimeout(timer);
	}

	return {
		/** A copy installed from GitHub. Nothing is shown anywhere for any other. */
		get eligible() {
			return status?.eligible ?? false;
		},
		/** Whether the start screen asks the question. */
		get ask() {
			return (status?.eligible ?? false) && status?.ask === true;
		},
		get checks() {
			return status?.checks ?? 'unanswered';
		},
		get lastCheckedMs() {
			return status?.lastCheckedMs ?? null;
		},
		/** The newer version on offer, `X.Y.Z`, or null. */
		get available() {
			return status?.eligible ? (status.available ?? null) : null;
		},
		load,
		schedule,
		check,
		setChecks,
		notNow,
		openReleaseNotes,
		dispose
	};
}
