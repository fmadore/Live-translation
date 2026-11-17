import { expect, it } from 'vitest';
import { flushSync } from 'svelte';
import { createOverlayCaptions, type CaptionLine } from './overlayCaptions.svelte';
import { DEFAULT_FILLER_WORDS } from '#lib/cleanSpeech.js';
import type { Caption, Origin } from '#lib/types.js';

function caption(origin: Origin, text: string, final = false): Caption {
	return { origin, turnId: 1, text, sourceText: '', final, startMs: 0, endMs: 0 };
}

// E3: a caption for one track lays out that track's row again and leaves the others as they
// were, rather than cleaning and cutting every track's text on every delta. Read from an
// effect, as the overlay page reads it: that is where a row's derivation is kept between reads.
it('leaves the other tracks’ rows as they were when one track changes', () => {
	let lines: CaptionLine[] = [];
	const stop = $effect.root(() => {
		const captions = createOverlayCaptions({
			layout: 'fit',
			hideFillers: true,
			fillerWords: DEFAULT_FILLER_WORDS,
			hold: 4,
			pace: 'immediate',
			width: 30,
			showOriginal: false
		});
		$effect(() => {
			lines = captions.lines;
		});
		captions.push(caption('microphone', 'Room.', true));
		captions.push(caption('system', 'Remote'));
		flushSync();
		const room = lines[1];
		captions.push(caption('system', 'Remote speaker'));
		flushSync();
		expect(lines[0].text).toBe('Remote speaker');
		expect(lines[1]).toBe(room);
		captions.dispose();
	});
	stop();
});
