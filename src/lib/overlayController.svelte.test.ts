// The remembered overlay placement through the controller, against Tauri's IPC mocks: the real
// `tauri.ts` runs, and the test plays a core with one overlay window on a known set of
// displays. What is checked is what the core is asked to do and what ends up in storage.

import { afterEach, beforeEach, expect, it } from 'vitest';
import { mockTauri, type TauriMock } from './testing/tauriMock';
import { createOverlayController } from './overlayController.svelte';
import { OVERLAY_GEOMETRY_KEY } from './overlayGeometry';
import type { Placement } from './profiles';

const LAPTOP = '0,0 2560x1600 150%';
const ROOM = '0,0 2560x1600 150%; 2560,0 1920x1080 100%';
const created: Placement = { x: 640, y: 720, width: 1280, height: 160 };
const strip: Placement = { x: 2608, y: 880, width: 1824, height: 160 };
const corner: Placement = { x: 40, y: 1300, width: 900, height: 200 };

let core: TauriMock;
/** The overlay window's rectangle as the fake core holds it. */
let rect: Placement;
let layout: string;

const stored = () => JSON.parse(localStorage.getItem(OVERLAY_GEOMETRY_KEY) ?? 'null') as unknown;
const store = (value: unknown) => localStorage.setItem(OVERLAY_GEOMETRY_KEY, JSON.stringify(value));

beforeEach(() => {
	localStorage.removeItem(OVERLAY_GEOMETRY_KEY);
	rect = { ...created };
	layout = ROOM;
	core = mockTauri();
	core.handle('display_layout', () => layout);
	core.handle('get_overlay_placement', () => rect);
	core.handle('set_overlay_placement', ({ placement }) => {
		rect = placement as Placement;
	});
});

afterEach(async () => {
	await core.dispose();
});

it('remembers the placement under the display layout when the operator presses Done', async () => {
	const overlay = createOverlayController();
	await overlay.toggleMoveOverlay();
	// Entering move mode is not a placement.
	expect(core.callsTo('get_overlay_placement')).toHaveLength(0);
	expect(stored()).toBeNull();
	rect = strip; // dragged onto the projector
	await overlay.toggleMoveOverlay();
	expect(overlay.moveOverlay).toBe(false);
	expect(stored()).toEqual([{ layout: ROOM, placement: strip }]);
});

it('remembers a placement locked from the overlay, but not one Escape put back', async () => {
	const overlay = createOverlayController();
	await overlay.toggleMoveOverlay();
	rect = strip;
	// Escape: the overlay restores the rectangle move mode found and says only that it left.
	rect = created;
	overlay.applyState({ interactive: false });
	expect(overlay.moveOverlay).toBe(false);
	await new Promise((resolve) => setTimeout(resolve, 0));
	expect(core.callsTo('get_overlay_placement')).toHaveLength(0);
	expect(stored()).toBeNull();

	await overlay.toggleMoveOverlay();
	rect = strip;
	// Enter, or the overlay's own Lock button.
	overlay.applyState({ interactive: false, placed: true });
	await expect.poll(stored).toEqual([{ layout: ROOM, placement: strip }]);
});

it('places a profile through the core and remembers where the core put it', async () => {
	const overlay = createOverlayController();
	// The core clamps what it is handed; what is remembered is where the window ended up.
	core.handle('set_overlay_placement', () => {
		rect = { ...corner, y: 1240 };
	});
	await overlay.applyPlacement(corner);
	expect(core.callsTo('set_overlay_placement')).toEqual([{ placement: corner }]);
	expect(stored()).toEqual([{ layout: ROOM, placement: { ...corner, y: 1240 } }]);
});

it('keeps one placement per layout, so each set of displays gets its own back', async () => {
	const overlay = createOverlayController();
	await overlay.applyPlacement(strip);
	layout = LAPTOP; // the projector is unplugged
	await overlay.applyPlacement(corner);
	expect(stored()).toEqual([
		{ layout: LAPTOP, placement: corner },
		{ layout: ROOM, placement: strip }
	]);

	layout = ROOM; // …and plugged back in at the next launch
	await createOverlayController().restorePlacement();
	expect(core.callsTo('set_overlay_placement').at(-1)).toEqual({ placement: strip });
	// Reopening on a layout is a use of it: it is the most recent again.
	expect(stored()).toEqual([
		{ layout: ROOM, placement: strip },
		{ layout: LAPTOP, placement: corner }
	]);
});

it('restores the most recent placement on displays it has not seen, through the core', async () => {
	store([
		{ layout: LAPTOP, placement: corner },
		{ layout: ROOM, placement: strip }
	]);
	layout = '0,0 1920x1080 100%';
	await createOverlayController().restorePlacement();
	// The core is what clamps it onto a display that is there.
	expect(core.callsTo('set_overlay_placement')).toEqual([{ placement: corner }]);
	// A placement it was not made for is not remembered as this layout's own.
	expect(stored()).toEqual([
		{ layout: LAPTOP, placement: corner },
		{ layout: ROOM, placement: strip }
	]);
});

it('restores the most recent placement when the core cannot name the layout', async () => {
	store([{ layout: ROOM, placement: strip }]);
	core.handle('display_layout', () => {
		throw new Error('No display available');
	});
	await createOverlayController().restorePlacement();
	expect(core.callsTo('set_overlay_placement')).toEqual([{ placement: strip }]);
});

it('leaves a first launch where the window was created, and ignores malformed storage', async () => {
	await createOverlayController().restorePlacement();
	localStorage.setItem(OVERLAY_GEOMETRY_KEY, '{"layout": broken');
	await createOverlayController().restorePlacement();
	store([{ layout: ROOM, placement: { x: 'left', y: 0, width: 0, height: 160 } }]);
	await createOverlayController().restorePlacement();
	expect(core.calls).toEqual([]);
	expect(rect).toEqual(created);
});

it('applies a profile only once a restore still under way has landed', async () => {
	store([{ layout: ROOM, placement: strip }]);
	let answer!: (signature: string) => void;
	core.handle('display_layout', () => new Promise<string>((resolve) => (answer = resolve)));
	const overlay = createOverlayController();
	const restoring = overlay.restorePlacement();
	const applying = overlay.applyPlacement(corner);
	await expect.poll(() => answer).toBeDefined();
	core.handle('display_layout', () => layout);
	answer(ROOM);
	await Promise.all([restoring, applying]);
	expect(core.callsTo('set_overlay_placement')).toEqual([
		{ placement: strip },
		{ placement: corner }
	]);
	expect(rect).toEqual(corner);
	expect(stored()).toEqual([{ layout: ROOM, placement: corner }]);
});

it('carries on when the placement cannot be read, and remembers nothing', async () => {
	core.handle('get_overlay_placement', () => {
		throw new Error('Overlay unavailable');
	});
	const overlay = createOverlayController();
	await overlay.toggleMoveOverlay();
	await overlay.toggleMoveOverlay();
	expect(overlay.moveOverlay).toBe(false);
	expect(stored()).toBeNull();
});
