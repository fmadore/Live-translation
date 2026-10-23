// Move mode, against Tauri's IPC mocks as the overlay window: the commands it sends are what is
// checked. Issue #137: a display that went during move mode must not get the window back, which
// the core now decides (`undo_overlay_move`). B4: every command the overlay sends is one its
// capability grants.

import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { mockTauri, type TauriMock } from '#lib/testing/tauriMock.js';
import capability from '../../../src-tauri/capabilities/overlay.json';
import { createOverlayPlacement } from './overlayPlacement.svelte';

/** Where move mode found the window: a strip along the bottom of the projector. */
const entry = { x: 2608, y: 880, width: 1824, height: 160 };

let core: TauriMock;

beforeEach(() => {
	core = mockTauri('overlay');
	core.handle('get_overlay_placement', () => entry);
	core.handle('undo_overlay_move', () => true);
});

afterEach(async () => {
	await core.dispose();
});

/** What `capabilities/overlay.json` lets this window invoke, as Tauri names the commands: app
 *  commands by name, core ones as `plugin:<plugin>|<command>`. */
function granted(): Set<string> {
	return new Set(
		capability.permissions.map((permission) => {
			const core = /^core:([a-z]+):allow-([a-z-]+)$/.exec(permission);
			if (core) return `plugin:${core[1]}|${core[2].replaceAll('-', '_')}`;
			expect(permission, 'an app command permission').toMatch(/^allow-[a-z-]+$/);
			return permission.slice('allow-'.length).replaceAll('-', '_');
		})
	);
}

/** Enter move mode and wait for the rect it records for Escape. */
async function enterMoveMode() {
	const placement = createOverlayPlacement();
	placement.setInteractive(true);
	await expect.poll(() => core.callsTo('get_overlay_placement')).toHaveLength(1);
	await new Promise((resolve) => setTimeout(resolve, 0));
	return placement;
}

it('asks the core to put the window back where move mode found it', async () => {
	const placement = await enterMoveMode();
	await placement.cancel();
	expect(core.callsTo('undo_overlay_move')).toEqual([{ entry }]);
	expect(core.callsTo('set_overlay_click_through')).toEqual([{ enabled: true }]);
	expect(placement.interactive).toBe(false);
	// Still as a cancel: nothing new was placed, so the operator is not told it was.
	await expect
		.poll(() => core.callsTo('report_overlay_state'))
		.toEqual([{ state: { interactive: false } }]);
});

it('leaves move mode all the same when the window cannot be put back', async () => {
	const placement = await enterMoveMode();
	core.handle('undo_overlay_move', () => {
		throw new Error('Overlay unavailable');
	});
	const logged = vi.spyOn(console, 'error').mockImplementation(() => {});
	try {
		await placement.cancel();
		expect(logged).toHaveBeenCalled();
	} finally {
		logged.mockRestore();
	}
	expect(core.callsTo('set_overlay_click_through')).toEqual([{ enabled: true }]);
	expect(placement.interactive).toBe(false);
});

it('locks into place and tells the operator window through the core', async () => {
	const placement = await enterMoveMode();
	await placement.lock();
	expect(placement.interactive).toBe(false);
	expect(core.callsTo('undo_overlay_move')).toEqual([]);
	await expect
		.poll(() => core.callsTo('report_overlay_state'))
		.toEqual([{ state: { interactive: false, placed: true } }]);
});

it('drags, nudges and snaps itself only through its own commands', async () => {
	const placement = await enterMoveMode();
	await placement.startDrag();
	await placement.nudge(-10, 1);
	await placement.snapToBottom(120);
	expect(core.callsTo('start_overlay_drag')).toHaveLength(1);
	expect(core.callsTo('nudge_overlay')).toEqual([{ dx: -10, dy: 1 }]);
	expect(core.callsTo('snap_overlay_to_bottom')).toEqual([{ stripHeight: 120 }]);
	await placement.lock();
	// Outside move mode a press on the stage is not a drag.
	await placement.startDrag();
	expect(core.callsTo('start_overlay_drag')).toHaveLength(1);
});

it('sends nothing its capability does not grant', async () => {
	const placement = await enterMoveMode();
	await placement.startDrag();
	await placement.nudge(1, 0);
	await placement.snapToBottom(120);
	await placement.cancel();
	placement.setInteractive(true);
	await expect.poll(() => core.callsTo('get_overlay_placement')).toHaveLength(2);
	await placement.lock();
	await expect.poll(() => core.callsTo('report_overlay_state')).toHaveLength(2);

	const allowed = granted();
	for (const { cmd } of core.calls) expect(allowed, cmd).toContain(cmd);
});
