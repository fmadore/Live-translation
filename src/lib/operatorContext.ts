// The two controllers nearly every part of the operator window reads: the overlay (its
// appearance, placement and visibility) and the pre-flight check (devices, the audio test,
// engine readiness). The page creates each once and provides them here, so a component four
// levels down asks for one instead of having it handed through every component in between.
//
// Each is provided as it is, the object its own module creates; this only carries it.

import { getContext, setContext } from 'svelte';
import type { OverlayController } from './overlayController.svelte';
import type { PreflightController } from './preflightController.svelte';

const OVERLAY = Symbol('overlay');
const PREFLIGHT = Symbol('preflight');

export interface OperatorContext {
	overlay: OverlayController;
	preflight: PreflightController;
}

/** Provide both controllers to every component under the caller. Call during the page's
 *  initialisation, as Svelte requires of a context. */
export function provideOperatorContext({ overlay, preflight }: OperatorContext): void {
	setContext(OVERLAY, overlay);
	setContext(PREFLIGHT, preflight);
}

function required<T>(key: symbol, name: string): T {
	const value = getContext<T | undefined>(key);
	// A component mounted outside the page (a test that forgot it, say) learns why at once,
	// rather than failing on the first click with "cannot read properties of undefined".
	if (value === undefined) throw new Error(`No ${name} controller in context`);
	return value;
}

/** The overlay controller the page provided. Call during component initialisation. */
export function overlayContext(): OverlayController {
	return required(OVERLAY, 'overlay');
}

/** The pre-flight controller the page provided. Call during component initialisation. */
export function preflightContext(): PreflightController {
	return required(PREFLIGHT, 'pre-flight');
}

/** The same context as a map, for `render(Component, { context })` in a component test. */
export function operatorContextMap(controllers: Partial<OperatorContext>): Map<symbol, unknown> {
	const map = new Map<symbol, unknown>();
	if (controllers.overlay) map.set(OVERLAY, controllers.overlay);
	if (controllers.preflight) map.set(PREFLIGHT, controllers.preflight);
	return map;
}
