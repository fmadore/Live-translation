import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createOverlayCaptions, tail, type ReadingSettings } from './overlayCaptions.svelte';
import type { Caption, Origin } from '$lib/types';

const fit: ReadingSettings = {
	layout: 'fit',
	hideFillers: false,
	hold: 4,
	pace: 'immediate',
	width: 30
};

function caption(origin: Origin, turnId: number, text: string, final = true): Caption {
	return { origin, turnId, text, sourceText: '', final, startMs: 0, endMs: 0 };
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

describe('overlay captions', () => {
	it('shows the remote speaker above the room, each with its own turn', () => {
		const c = createOverlayCaptions(fit);
		c.push(caption('microphone', 1, 'Room.'));
		c.push(caption('system', 1, 'Remote.'));
		expect(c.lines.map((line) => [line.origin, line.text])).toEqual([
			['system', 'Remote.'],
			['microphone', 'Room.']
		]);
		c.dispose();
	});

	it('keeps a finished turn as the lead-in to the next one in Fit window', () => {
		const c = createOverlayCaptions(fit);
		c.push(caption('system', 1, 'First sentence.'));
		c.push(caption('system', 2, 'Second', false));
		expect(c.lines).toEqual([
			{ origin: 'system', lead: 'First sentence.', text: 'Second', interim: true }
		]);
		c.dispose();
	});

	it('clears a finished caption after the hold, and a stalled interim sooner', () => {
		const c = createOverlayCaptions({ ...fit, hold: 8 });
		c.push(caption('system', 1, 'Final.'));
		c.push(caption('microphone', 1, 'Stalled', false));
		vi.advanceTimersByTime(3000);
		expect(c.lines.map((line) => line.origin)).toEqual(['system']);
		vi.advanceTimersByTime(5000);
		expect(c.lines).toEqual([]);
		c.dispose();
	});

	it('rearms every expiry when the hold changes', () => {
		const c = createOverlayCaptions(fit);
		c.push(caption('system', 1, 'Final.'));
		vi.advanceTimersByTime(3000);
		c.setHold(10);
		vi.advanceTimersByTime(9000);
		expect(c.lines).toHaveLength(1);
		vi.advanceTimersByTime(1000);
		expect(c.lines).toEqual([]);
		c.dispose();
	});

	it('trims a streaming turn to the Compact budget and spends only real leftover room on a lead', () => {
		const c = createOverlayCaptions({ ...fit, layout: 'compact', width: 20 });
		const long = 'word '.repeat(200).trim();
		c.push(caption('system', 1, 'A short earlier sentence that is long enough to show.'));
		c.push(caption('system', 2, 'Short', false));
		expect(c.lines[0].lead.length).toBeGreaterThan(0);
		c.push(caption('system', 2, long, false));
		expect(c.lines[0].text.startsWith('… ')).toBe(true);
		expect(c.lines[0].lead).toBe('');
		c.dispose();
	});

	it('retains Stable reading across pauses and clears it when the session stops', () => {
		const c = createOverlayCaptions({ ...fit, layout: 'stable' });
		c.push(caption('system', 1, 'One.'));
		c.push(caption('system', 2, 'Two.'));
		vi.advanceTimersByTime(60_000);
		expect(c.lines[0]).toMatchObject({ lead: ' One.', text: 'Two.' });
		c.status({ state: 'idle', origin: 'system' });
		expect(c.lines).toHaveLength(1);
		c.status({ state: 'idle' });
		expect(c.lines).toEqual([]);
		c.dispose();
	});

	it('trims Stable context at the offset the caption line measured', () => {
		const c = createOverlayCaptions({ ...fit, layout: 'stable' });
		c.push(caption('system', 1, 'Old line.'));
		c.push(caption('system', 2, 'Now.'));
		c.trimStable('system', ' Old line.'.length);
		expect(c.lines[0].lead).toBe('');
		c.dispose();
	});

	it('cleans Stable context once as turns join it, so a later toggle only affects new turns', () => {
		const c = createOverlayCaptions({ ...fit, layout: 'stable', hideFillers: true });
		c.push(caption('system', 1, 'Um, first.'));
		c.push(caption('system', 2, 'Second.'));
		c.setHideFillers(false);
		c.push(caption('system', 3, 'Third.'));
		expect(c.lines[0].lead).toBe(' First. Second.');
		c.dispose();
	});

	it('restores expiry when leaving Stable reading', () => {
		const c = createOverlayCaptions({ ...fit, layout: 'stable' });
		c.push(caption('system', 1, 'Kept.'));
		vi.advanceTimersByTime(10_000);
		c.setLayout('fit');
		expect(c.lines).toHaveLength(1);
		vi.advanceTimersByTime(4000);
		expect(c.lines).toEqual([]);
		c.dispose();
	});

	it('holds interim text under the steady pace until it is flushed', () => {
		const c = createOverlayCaptions({ ...fit, pace: 'steady' });
		c.push(caption('system', 1, 'Streaming', false));
		expect(c.lines).toEqual([]);
		vi.advanceTimersByTime(450);
		expect(c.lines.map((line) => line.text)).toEqual(['Streaming']);
		c.dispose();
	});
});

describe('tail', () => {
	it('keeps short text whole and cuts long text at a nearby word boundary', () => {
		expect(tail('  a   short   line ', 40)).toBe('a short line');
		expect(tail('alpha beta gamma delta', 10)).toBe('… delta');
	});
});
