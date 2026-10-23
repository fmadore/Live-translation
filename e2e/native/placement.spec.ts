// The overlay reopens where it was placed. It is placed as an operator places it — move mode,
// a new rectangle, Done — then the app is quit and started again on the same profile, and the
// core reports that rectangle once the operator window has restored it. A runner has one
// display, so this is the same-layout case; other layouts, the eight-layout cap and mixed DPI
// are left to the unit tests and the manual checklist in docs/caption-layout.md.
//
// And Move overlay finds an overlay stranded off every display (issue #137). A runner cannot
// unplug a monitor, so the overlay strands itself, as a projector pulled out from under it
// would leave it; what is checked is that entering move mode brings it back onto the one
// display there is. The display-change event itself is left to the manual checklist.

import type { Page } from '@playwright/test';
import { expect, test } from './fixtures';

interface Rect {
	x: number;
	y: number;
	width: number;
	height: number;
}

interface Bounds {
	position: { x: number; y: number };
	size: { width: number; height: number };
}

interface Monitor extends Bounds {
	workArea: Bounds;
}

type TauriWindow = {
	__TAURI_INTERNALS__: { invoke(cmd: string, args: unknown): Promise<unknown> };
};

/** A command sent from the page as the app sends it, so the window's capability applies. */
function invoke<T>(page: Page, cmd: string, args: Record<string, unknown> = {}): Promise<T> {
	return page.evaluate(
		([cmd, args]) => (window as unknown as TauriWindow).__TAURI_INTERNALS__.invoke(cmd, args),
		[cmd, args] as const
	) as Promise<T>;
}

test('the overlay reopens where it was last placed', async ({ app }) => {
	const created = await invoke<Rect>(app.operator, 'get_overlay_placement');
	const { workArea } = await invoke<Monitor>(app.operator, 'plugin:window|primary_monitor');
	// Well inside the work area, so the core's clamp keeps it as it is, and nowhere near the
	// centre the window is created at.
	const placed: Rect = {
		x: workArea.position.x + 40,
		y: workArea.position.y + 60,
		width: Math.min(640, workArea.size.width - 80),
		height: 200
	};
	expect(placed).not.toEqual(created);

	await app.operator.getByRole('button', { name: 'Move overlay' }).click();
	// Only once move mode is on, as an operator can only drag then: entering it first makes sure
	// the overlay is reachable, and on a runner narrower than the created overlay that clamps
	// and moves it — a move that would land on top of a placement sent while it was under way.
	const done = app.operator.getByRole('button', { name: 'Finish placing the overlay' });
	await expect(done).toBeVisible();
	// The drag: the window moves and resizes itself in move mode, through the placement command.
	await invoke(app.operator, 'set_overlay_placement', { placement: placed });
	await done.click();
	await expect
		.poll(() => app.operator.evaluate(() => localStorage.getItem('overlay.geometry')), {
			message: 'the placement remembered'
		})
		.toContain(`"placement":${JSON.stringify(placed)}`);

	await app.relaunch();

	await expect
		.poll(() => invoke<Rect>(app.operator, 'get_overlay_placement'), {
			message: 'the overlay back where it was placed'
		})
		.toEqual(placed);
});

/** Whether `rect` lies wholly inside `area`. */
function inside(rect: Rect, { position, size }: Bounds): boolean {
	return (
		rect.x >= position.x &&
		rect.y >= position.y &&
		rect.x + rect.width <= position.x + size.width &&
		rect.y + rect.height <= position.y + size.height
	);
}

test('Move overlay brings back an overlay stranded off every display', async ({ app }) => {
	const monitors = await invoke<Monitor[]>(app.operator, 'plugin:window|available_monitors');
	const { workArea } = await invoke<Monitor>(app.operator, 'plugin:window|primary_monitor');
	// Well past the right-hand edge of everything, as if the display it was on had gone. Sent
	// from the overlay, whose capability lets it move itself and nothing else, in physical
	// pixels from where it is.
	const right = Math.max(...monitors.map(({ position, size }) => position.x + size.width));
	const at = await invoke<Rect>(app.operator, 'get_overlay_placement');
	await invoke(app.overlay, 'nudge_overlay', {
		dx: right + 4000 - at.x,
		dy: workArea.position.y + 200 - at.y
	});
	await expect
		.poll(async () => (await invoke<Rect>(app.operator, 'get_overlay_placement')).x, {
			message: 'the overlay stranded'
		})
		.toBeGreaterThan(right);

	await app.operator.getByRole('button', { name: 'Move overlay' }).click();
	await expect
		.poll(async () => inside(await invoke<Rect>(app.operator, 'get_overlay_placement'), workArea), {
			message: 'the overlay back inside the work area'
		})
		.toBe(true);
	// Shown, as move mode always shows it: on a window the operator can see this time.
	expect(
		await invoke<boolean>(app.operator, 'plugin:window|is_visible', { label: 'overlay' })
	).toBe(true);

	await app.operator.getByRole('button', { name: 'Finish placing the overlay' }).click();
	await expect(app.operator.getByRole('button', { name: 'Move overlay' })).toBeVisible();
});

// B4: the overlay is the window an audience looks at, so it may move itself and report what
// happened on it, and nothing else. Without `emit` it cannot pass for the core (a tray command
// that stops the session, captions that reach the transcript), and without the core window
// setters it cannot move the operator window.
test('the overlay can neither emit events nor move another window', async ({ app }) => {
	await expect(
		invoke(app.overlay, 'plugin:event|emit', { event: 'tray-command', payload: 'stop-session' })
	).rejects.toBeDefined();
	await expect(
		invoke(app.overlay, 'plugin:window|set_position', {
			label: 'operator',
			value: { Physical: { x: 0, y: 0 } }
		})
	).rejects.toBeDefined();
	// What it is granted still works.
	await invoke(app.overlay, 'report_overlay_state', { state: { fontSize: 40 } });
});
