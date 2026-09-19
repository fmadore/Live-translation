import type { Caption, Origin } from './types';

export type CaptionPace = 'immediate' | 'steady';
export const HOLD_KEY = 'overlay.holdSeconds';
export const PACE_KEY = 'overlay.pace';
export function holdSeconds(value: unknown): number {
	return typeof value === 'number' && Number.isFinite(value)
		? Math.max(2, Math.min(30, Math.round(value)))
		: 4;
}
export function loadHoldSeconds(): number {
	const value = typeof localStorage !== 'undefined' ? localStorage.getItem(HOLD_KEY) : null;
	return value === null ? 4 : holdSeconds(Number(value));
}
export function loadPace(): CaptionPace {
	return typeof localStorage !== 'undefined' && localStorage.getItem(PACE_KEY) === 'steady'
		? 'steady'
		: 'immediate';
}

/** Throttle rather than debounce: continuous speech must still appear every 450ms.
 * Final text and turn transitions flush immediately; origins never share a timer. */
export function createCaptionPresenter(show: (caption: Caption) => void, pace: () => CaptionPace) {
	const pending: Partial<Record<Origin, Caption>> = {};
	const timers: Partial<Record<Origin, ReturnType<typeof setTimeout>>> = {};
	function flush(origin: Origin) {
		clearTimeout(timers[origin]);
		delete timers[origin];
		const caption = pending[origin];
		delete pending[origin];
		if (caption) show(caption);
	}
	return {
		push(caption: Caption) {
			if (pending[caption.origin]?.turnId !== caption.turnId) flush(caption.origin);
			pending[caption.origin] = caption;
			if (pace() === 'immediate' || caption.final) flush(caption.origin);
			else if (!timers[caption.origin])
				timers[caption.origin] = setTimeout(() => flush(caption.origin), 450);
		},
		flush() {
			flush('microphone');
			flush('system');
		},
		clear(origin?: Origin) {
			for (const key of origin ? [origin] : (['microphone', 'system'] as const)) {
				clearTimeout(timers[key]);
				delete timers[key];
				delete pending[key];
			}
		}
	};
}
