// Move mode for the overlay window: entering it, nudging and snapping the window, and the two
// ways out — lock the region into place, or cancel and put the window back where it was.

// Every window operation goes through an overlay-only app command that acts on this window
// alone (B4): the core window API's setters take any window's label, so the overlay is not
// granted them, nor the getters it no longer needs.

import { api, isTauri } from '#lib/tauri.js';
import { writeStored } from '#lib/persisted.js';
import { OVERLAY_PLACED_KEY } from '#lib/appearance.js';
import type { Placement } from '#lib/profiles.js';
import type { OverlayStateMsg } from '#lib/types.js';

/** Physical pixels, straight off the window: restoring what was read needs no conversion. The
 *  core's `Placement`. */
type Geometry = Placement;

export type OverlayCommand =
	| { kind: 'bump'; delta: number }
	| { kind: 'lock' }
	| { kind: 'cancel' }
	| { kind: 'nudge'; dx: number; dy: number };

/**
 * What a key does in the overlay. The overlay only has a keyboard in move mode, and that is
 * exactly when it may be covering the operator window's own "Done moving" button (issue #11)
 * — so both ways out have to be reachable from here. The +/− size keys work in either mode.
 */
export function overlayKeyCommand(
	key: { key: string; shiftKey: boolean; onButton: boolean },
	interactive: boolean
): OverlayCommand | null {
	if (key.key === '+' || key.key === '=') return { kind: 'bump', delta: 2 };
	if (key.key === '-') return { kind: 'bump', delta: -2 };
	if (!interactive) return null;
	const step = key.shiftKey ? 10 : 1;
	switch (key.key) {
		// A focused toolbar button already answers Enter itself; hijacking it would run the
		// button and the shortcut at once.
		case 'Enter':
			return key.onButton ? null : { kind: 'lock' };
		case 'Escape':
			return { kind: 'cancel' };
		case 'ArrowLeft':
			return { kind: 'nudge', dx: -step, dy: 0 };
		case 'ArrowRight':
			return { kind: 'nudge', dx: step, dy: 0 };
		case 'ArrowUp':
			return { kind: 'nudge', dx: 0, dy: -step };
		case 'ArrowDown':
			return { kind: 'nudge', dx: 0, dy: step };
		default:
			return null;
	}
}

export function createOverlayPlacement(port = api) {
	// Click-through is off and the whole stage becomes a drag handle, so the window can be
	// dragged and resized into place.
	let interactive = $state(false);
	// Where the window sat when move mode was entered, so Escape can undo the whole move.
	// Not reactive — nothing renders it.
	let entryGeometry: Geometry | null = null;

	/** Remember the current window rect so a cancelled move can be undone. The outer position
	 *  and the inner size: the pair that can be handed straight back. */
	async function snapshotGeometry() {
		if (!isTauri()) {
			entryGeometry = null;
			return;
		}
		try {
			entryGeometry = await port.getOverlayPlacement();
		} catch (err) {
			console.error('Could not record the overlay geometry', err);
			entryGeometry = null;
		}
	}

	/** Arrow-key nudge, in whole physical pixels — finer than a mouse drag can manage. */
	async function nudge(dx: number, dy: number) {
		if (!isTauri()) return;
		try {
			await port.nudgeOverlay(dx, dy);
		} catch (err) {
			console.error('Nudge failed', err);
		}
	}

	/** Stretch the region across the presentation display, sitting on the bottom margin, as a
	 *  strip `stripHeight` logical pixels tall. The core works out the strip; see
	 *  `bottom_strip` in `placement.rs`. */
	async function snapToBottom(stripHeight: number) {
		if (!isTauri()) return;
		try {
			await port.snapOverlayToBottom(stripHeight);
		} catch (err) {
			console.error('Snap to bottom failed', err);
		}
	}

	/** Drag the window with the mouse, from the stage itself: what `data-tauri-drag-region`
	 *  did, through a command that can move only this window. */
	async function startDrag() {
		if (!interactive || !isTauri()) return;
		try {
			await port.startOverlayDrag();
		} catch (err) {
			console.error('Could not start dragging the overlay', err);
		}
	}

	/** Tell the operator window. Fire-and-forget: a report that never arrives costs the
	 *  operator's toggle one extra click, not the overlay its state. */
	function report(state: OverlayStateMsg) {
		if (!isTauri()) return;
		void port
			.reportOverlayState(state)
			.catch((err: unknown) => console.error('Could not tell the operator window', err));
	}

	/** Leave move mode: click-through back on, and both windows told the region is placed. */
	async function lock() {
		try {
			// Also re-enables no-activate on the Rust side, so raising can't steal focus.
			await port.setOverlayClickThrough(true);
		} catch (err) {
			console.error('Failed to restore overlay click-through', err);
		}
		entryGeometry = null;
		interactive = false;
		writeStored(OVERLAY_PLACED_KEY, 'true');
		report({ interactive: false, placed: true });
	}

	/** Move and size the window back to `geo`, unless that is on a display that has gone: a
	 *  projector pulled out during move mode takes it away, and the core has already brought the
	 *  window onto a screen that is there. The core decides, as it decides a rescue. */
	async function restoreGeometry(geo: Geometry) {
		try {
			await port.undoOverlayMove(geo);
		} catch (err) {
			// A failed restore must not trap the operator in move mode — carry on and exit.
			console.error('Could not restore the overlay geometry', err);
		}
	}

	/** Put the window back where move mode found it and leave without placing it. Where it was
	 *  may be on a display that has gone since; then it stays where it is, and the move is
	 *  abandoned all the same. */
	async function cancel() {
		const geo = entryGeometry;
		if (geo && isTauri()) await restoreGeometry(geo);
		try {
			await port.setOverlayClickThrough(true);
		} catch (err) {
			console.error('Failed to restore overlay click-through', err);
		}
		entryGeometry = null;
		interactive = false;
		// No `placed`: the move was abandoned, so the pre-flight check must still ask for it.
		report({ interactive: false });
	}

	return {
		get interactive() {
			return interactive;
		},
		/** The operator window turned move mode on or off. Entering records the rect first,
		 *  so Escape has something to restore. */
		setInteractive(next: boolean) {
			if (next && !interactive) void snapshotGeometry();
			interactive = next;
		},
		nudge,
		snapToBottom,
		startDrag,
		lock,
		cancel
	};
}
