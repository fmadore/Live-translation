// The overlay reopens where it was placed. It is placed as an operator places it — move mode,
// a new rectangle, Done — then the app is quit and started again on the same profile, and the
// core reports that rectangle once the operator window has restored it. A runner has one
// display, so this is the same-layout case; other layouts, the eight-layout cap and mixed DPI
// are left to the unit tests and the manual checklist in docs/caption-layout.md.

import type { Page } from '@playwright/test';
import { expect, test } from './fixtures';

interface Rect {
	x: number;
	y: number;
	width: number;
	height: number;
}

interface Monitor {
	workArea: { position: { x: number; y: number }; size: { width: number; height: number } };
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
	// The drag: the window moves and resizes itself in move mode, through the placement command.
	await invoke(app.operator, 'set_overlay_placement', { placement: placed });
	await app.operator.getByRole('button', { name: 'Finish placing the overlay' }).click();
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
