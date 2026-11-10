import { afterEach, describe, expect, it, vi } from 'vitest';
import { listenAll } from './coreEvents';

/** A subscription whose answer the test gives when it chooses. */
function pending() {
	let resolve!: (unlisten: () => void) => void;
	let reject!: (error: unknown) => void;
	const promise = new Promise<() => void>((yes, no) => {
		resolve = yes;
		reject = no;
	});
	return { promise, resolve, reject, unlisten: vi.fn() };
}

afterEach(() => vi.restoreAllMocks());

describe('listenAll', () => {
	it('is ready once every subscription is in place, and stops them all', async () => {
		const a = pending();
		const b = pending();
		const events = listenAll([a.promise, b.promise]);
		a.resolve(a.unlisten);
		b.resolve(b.unlisten);
		expect(await events.ready).toBe(true);
		events.stop();
		events.stop();
		expect(a.unlisten).toHaveBeenCalledOnce();
		expect(b.unlisten).toHaveBeenCalledOnce();
	});

	// Review B17: one refused `listen` used to keep every other listener subscribed past the
	// window's teardown, with an unhandled rejection.
	it('stops the others when one subscription is refused, without rejecting', async () => {
		const error = vi.spyOn(console, 'error').mockImplementation(() => {});
		const a = pending();
		const b = pending();
		const events = listenAll([a.promise, b.promise]);
		a.resolve(a.unlisten);
		b.reject(new Error('event.listen not allowed'));
		expect(await events.ready).toBe(false);
		expect(error).toHaveBeenCalledOnce();
		events.stop();
		expect(a.unlisten).toHaveBeenCalledOnce();
	});

	it('stops a subscription that arrives after the window went away', async () => {
		const a = pending();
		const events = listenAll([a.promise]);
		events.stop();
		a.resolve(a.unlisten);
		expect(await events.ready).toBe(false);
		expect(a.unlisten).toHaveBeenCalledOnce();
	});

	it('stops the rest when one unlisten throws', async () => {
		vi.spyOn(console, 'error').mockImplementation(() => {});
		const a = pending();
		const b = pending();
		const events = listenAll([a.promise, b.promise]);
		a.resolve(() => {
			throw new Error('already gone');
		});
		b.resolve(b.unlisten);
		await events.ready;
		events.stop();
		expect(b.unlisten).toHaveBeenCalledOnce();
	});
});
