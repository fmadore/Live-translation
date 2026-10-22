import { expect, it } from 'vitest';
import { activity } from './liveActivity';
it('distinguishes silence, audio, recent text and missing captions without calling silence a failure', () => {
	expect(activity('running', 20000, 1000, 0, 0)).toBe('listening');
	expect(activity('running', 5000, 1000, 4900, 0)).toBe('audio');
	expect(activity('running', 20000, 1000, 19000, 0)).toBe('stale');
	expect(activity('running', 20000, 1000, 19000, 19500)).toBe('captions');
	expect(activity('reconnecting', 20000, 1000, 19000, 19500)).toBe('reconnecting');
	expect(activity('error', 20000, 1000, 19000, 19500)).toBe('error');
});

// Review U6: audio arrives during a pause or a slow reconnect too, and that time says nothing
// about captions going missing. Counted from when the source last came to be running.
it('waits fifteen seconds from a resume or a reconnect before calling captions missing', () => {
	// Running again for five seconds after a long break, the last caption from before it.
	expect(activity('running', 300_000, 295_000, 299_500, 60_000)).toBe('audio');
	expect(activity('running', 310_000, 295_000, 309_500, 60_000)).toBe('stale');
	// A caption since the resume counts from that caption instead.
	expect(activity('running', 312_000, 295_000, 311_500, 300_000)).toBe('audio');
});
