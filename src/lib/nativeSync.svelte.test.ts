// The operator window's mirrors to the core and the overlay, against Tauri's IPC mocks: the real
// `tauri.ts` runs, the test plays the core, and what is checked is what the window sends.

import { afterEach, beforeEach, expect, it } from 'vitest';
import { flushSync } from 'svelte';
import { mockTauri, type TauriMock } from './testing/tauriMock';
import { createOverlayController } from './overlayController.svelte';
import { syncNative } from './nativeSync.svelte';
import type { SessionClock } from './sessionClock.svelte';
import { applyStatus, options } from './stores';
import { on } from './tauri';
import type { OverlayConfig } from './types';

let core: TauriMock;
let stop: (() => void) | undefined;
let unlisten: (() => void) | undefined;

beforeEach(() => {
	core = mockTauri();
});

afterEach(async () => {
	stop?.();
	unlisten?.();
	stop = unlisten = undefined;
	await core.dispose();
});

/** The operator page's part: a controller, its `overlay-state` listener and the mirrors. */
async function mount(clock = { elapsedMs: 0 } as SessionClock) {
	const overlay = createOverlayController();
	unlisten = await on.overlayState(overlay.applyState);
	stop = $effect.root(() => syncNative({ desktop: true, overlay, clock }));
	flushSync();
	return overlay;
}

const lastTray = () => core.callsTo('set_tray_state').at(-1);

/** Every `overlay-config` push the operator window sends, as the overlay would hear them. */
async function overlayPushes() {
	const pushes: OverlayConfig[] = [];
	const stopListening = await on.overlayConfig((config) => pushes.push(config));
	const previous = unlisten;
	unlisten = () => {
		stopListening();
		previous?.();
	};
	return pushes;
}

// U5: the overlay reports ready once its listeners are in place, and the operator answers with
// everything the overlay may have missed by loading late.
it('answers the overlay being ready with a whole configuration', async () => {
	options.update((o) => ({ ...o, mode: 'translate', targetLanguage: 'fr' }));
	const pushes = await overlayPushes();
	const overlay = await mount();
	// The push on mount, before the overlay was listening.
	await expect.poll(() => pushes.length).toBeGreaterThan(0);
	pushes.length = 0;

	await core.emit('overlay-state', { ready: true });
	await expect.poll(() => pushes.length).toBe(1);
	expect(pushes[0]).toMatchObject({
		locale: 'en',
		captionLanguage: 'fr',
		interactive: false,
		moveExitsSeen: 0
	});
	expect(pushes[0].fontSize).toEqual(expect.any(Number));
	expect(overlay.moveOverlay).toBe(false);

	// Anything else on the channel is not a reason to push.
	await core.emit('overlay-state', { fontSize: 40 });
	await new Promise((resolve) => setTimeout(resolve, 20));
	expect(pushes).toHaveLength(1);
});

it('carries move mode in the answer to a reloaded overlay', async () => {
	const pushes = await overlayPushes();
	const overlay = await mount();
	await overlay.toggleMoveOverlay();
	expect(overlay.moveOverlay).toBe(true);
	pushes.length = 0;
	await core.emit('overlay-state', { ready: true });
	await expect.poll(() => pushes.at(-1)).toMatchObject({ interactive: true });
});

// U5: the overlay's own Lock and a push sent before the operator heard of it cross on the way.
it('hands back the count of the overlay exits it has heard of', async () => {
	const pushes = await overlayPushes();
	const overlay = await mount();
	await overlay.toggleMoveOverlay();
	await core.emit('overlay-state', { interactive: false, placed: true, moveExits: 1 });
	await expect.poll(() => overlay.moveOverlay).toBe(false);
	pushes.length = 0;
	overlay.setFont(44);
	await expect.poll(() => pushes.at(-1)).toMatchObject({ moveExitsSeen: 1, interactive: false });
});

// B3: Alt+F4 on the overlay hides it in the core, which announces it; the tray's entry has to
// offer Show as well as the operator's toggle.
it('tells the tray the overlay is hidden once the core says so', async () => {
	const overlay = await mount();
	await expect.poll(lastTray).toMatchObject({ overlayVisible: true });
	await core.emit('overlay-state', { visible: false });
	await expect.poll(() => overlay.overlayVisible).toBe(false);
	flushSync();
	await expect.poll(lastTray).toMatchObject({ overlayVisible: false });
});

// Review E3: the tray's status line used to carry the clock to the second, so the whole tray
// state crossed IPC every second of a session. It now gives whole minutes, sent as they change.
it('tells the tray how long the session has run once a minute, not every second', async () => {
	let elapsedMs = $state(0);
	const clock = {
		get elapsedMs() {
			return elapsedMs;
		}
	} as SessionClock;
	applyStatus({ state: 'running', origin: 'microphone' });
	try {
		await mount(clock);
		await expect.poll(lastTray).toMatchObject({ labels: { status: 'Live · 0 min' } });
		const pushes = () => core.callsTo('set_tray_state').length;
		const before = pushes();
		for (const seconds of [1, 2, 30, 59]) {
			elapsedMs = seconds * 1000;
			flushSync();
		}
		await new Promise((resolve) => setTimeout(resolve, 20));
		expect(pushes()).toBe(before);

		elapsedMs = 60_000;
		flushSync();
		await expect.poll(lastTray).toMatchObject({ labels: { status: 'Live · 1 min' } });
		expect(pushes()).toBe(before + 1);
		elapsedMs = 61 * 60_000;
		flushSync();
		await expect.poll(lastTray).toMatchObject({ labels: { status: 'Live · 1 h 01 min' } });
	} finally {
		applyStatus({ state: 'idle' });
	}
});
