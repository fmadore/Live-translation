// Subscribing to the core's events from a component (review B17, S9). `listen` answers with a
// promise, so a window that goes away before it settles, or a subscription that is refused,
// has to be handled at every call site; this does it once.

/** A group of subscriptions ended together. */
export interface CoreListeners {
	/** True once every subscription is in place, false if one was refused or the group was
	 *  stopped first. Never rejects. */
	ready: Promise<boolean>;
	/** End every subscription: those in place now, and those still being set up as they
	 *  arrive. Safe to call more than once. */
	stop: () => void;
}

/** Hold `subscriptions` (each an `on.*` call) until `stop`. A refused one is logged and leaves
 *  the others subscribed, and stopped with the group: one rejection no longer kept every other
 *  listener alive past the window's teardown. */
export function listenAll(subscriptions: Promise<() => void>[]): CoreListeners {
	let stopped = false;
	const held: (() => void)[] = [];
	const release = (unlisten: () => void) => {
		try {
			unlisten();
		} catch (error) {
			console.error('Could not stop listening to the core', error);
		}
	};
	const ready = Promise.all(
		subscriptions.map((subscription) =>
			subscription.then(
				(unlisten) => {
					// Unmounted before `listen` answered: nothing will ever stop this one later.
					if (stopped) release(unlisten);
					else held.push(unlisten);
					return true;
				},
				(error: unknown) => {
					console.error('Could not listen to the core', error);
					return false;
				}
			)
		)
	).then((results) => !stopped && results.every(Boolean));
	return {
		ready,
		stop: () => {
			stopped = true;
			for (const unlisten of held.splice(0)) release(unlisten);
		}
	};
}
