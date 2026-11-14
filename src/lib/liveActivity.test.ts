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

// First run on a real Mac: a refused System Audio Recording permission does not fail the tap,
// it delivers zeros. The core marks that (sound playing, only silence arriving), and the row
// says so instead of calling it a quiet room.
it('tells a source the system keeps silent apart from a quiet room', () => {
	expect(activity('running', 20000, 1000, 0, 0, 19500)).toBe('silenced');
	// The mark fades like any other: three seconds without one is a quiet room again.
	expect(activity('running', 20000, 1000, 0, 0, 16000)).toBe('listening');
	// Real sound or captions always win: the permission was granted after all.
	expect(activity('running', 20000, 15000, 19800, 0, 19500)).toBe('audio');
	expect(activity('running', 20000, 1000, 0, 19800, 19500)).toBe('captions');
	expect(activity('error', 20000, 1000, 0, 0, 19500)).toBe('error');
});
