import { describe, expect, it } from 'vitest';
import { canFlipDirection } from './types';

// The operator UI used to offer "Flip mid-session with F2" over a handler that refused for
// the whole of a session (issue #21). Keeping the rule in one tested predicate is what stops
// the shortcut and the copy drifting apart again.
describe('canFlipDirection', () => {
	it('allows a swap while the operator is still setting a translation up', () => {
		expect(canFlipDirection('translate', false)).toBe(true);
	});

	it('refuses once controls are locked, which is the whole of a running session', () => {
		expect(canFlipDirection('translate', true)).toBe(false);
	});

	it('refuses in subtitle mode, which has no direction to flip', () => {
		expect(canFlipDirection('transcribe', false)).toBe(false);
		expect(canFlipDirection('transcribe', true)).toBe(false);
	});
});
