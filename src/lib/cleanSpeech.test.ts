import { describe, expect, it } from 'vitest';
import { cleanSpeech } from './cleanSpeech';

describe('display-only conservative hesitation cleanup', () => {
	it('cleans the requested sentence and its punctuation', () => {
		expect(cleanSpeech('Um, we need to, uh, validate the configuration before Monday.')).toBe(
			'We need to validate the configuration before Monday.'
		);
		expect(cleanSpeech('Euh, nous devons, heu, valider.')).toBe('Nous devons valider.');
		expect(cleanSpeech('um, uh, erm, hmm.')).toBe('');
	});
	it('preserves meaningful words, abbreviations, substrings and quotations', () => {
		const text =
			'So, well, I like the umbrella, human rhythm and HMM. “um” means hesitation; uh-oh!';
		expect(cleanSpeech(text)).toBe(text);
		expect(cleanSpeech('So, um, I think we should, uh, probably validate this.')).toBe(
			'So I think we should probably validate this.'
		);
		expect(cleanSpeech('ähm umami euhorie')).toBe('ähm umami euhorie');
	});
	it('waits for a token boundary in partial captions', () => {
		expect(cleanSpeech('Take the um', false)).toBe('Take the um');
		expect(cleanSpeech('Take the umbrella', false)).toBe('Take the umbrella');
		expect(cleanSpeech('Take, um, the umbrella', false)).toBe('Take the umbrella');
	});
	it('does not mutate the raw caption', () => {
		const caption = { text: 'Um, hello.', sourceText: 'Euh, bonjour.' };
		expect(cleanSpeech(caption.text)).toBe('Hello.');
		expect(caption).toEqual({ text: 'Um, hello.', sourceText: 'Euh, bonjour.' });
	});
});
