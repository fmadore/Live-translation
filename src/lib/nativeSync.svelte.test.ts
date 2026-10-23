// The operator window's mirrors to the core and the overlay, against Tauri's IPC mocks: the real
// `tauri.ts` runs, the test plays the core, and what is checked is what the window sends.

import { afterEach, beforeEach, expect, it } from 'vitest';
import { flushSync } from 'svelte';
import { mockTauri, type TauriMock } from './testing/tauriMock';
import { createOverlayController } from './overlayController.svelte';
import { syncNative } from './nativeSync.svelte';
import type { SessionClock } from './sessionClock.svelte';
import { on } from './tauri';

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
async function mount() {
	const overlay = createOverlayController();
	unlisten = await on.overlayState(overlay.applyState);
	const clock = { elapsed: '00:00' } as SessionClock;
	stop = $effect.root(() => syncNative({ desktop: true, overlay, clock }));
	flushSync();
	return overlay;
}

const lastTray = () => core.callsTo('set_tray_state').at(-1);

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
