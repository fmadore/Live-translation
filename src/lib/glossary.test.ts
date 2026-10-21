import { describe, expect, it } from 'vitest';
import {
	addGlossaryTerm,
	MAX_GLOSSARY_TERM_LENGTH,
	MAX_GLOSSARY_TERMS,
	normalizeGlossary,
	removeGlossaryTerm
} from './glossary';

/** `count` distinct terms. */
const terms = (count: number) => Array.from({ length: count }, (_, i) => `Term ${i}`);

describe('adding a name or term', () => {
	it('appends it trimmed, spaces inside kept', () => {
		expect(addGlossaryTerm(['UNESCO'], '  Université Joseph Ki-Zerbo ')).toEqual({
			list: ['UNESCO', 'Université Joseph Ki-Zerbo'],
			term: 'Université Joseph Ki-Zerbo'
		});
	});

	it('trims every space the core would, including a next-line character', () => {
		expect(addGlossaryTerm([], ' 　Ouagadougou\u0085\n')).toMatchObject({
			term: 'Ouagadougou'
		});
	});

	it('composes accents, so a decomposed spelling is the same entry as a composed one', () => {
		const added = addGlossaryTerm([], 'Amélie');
		expect(added).toEqual({ list: ['Amélie'], term: 'Amélie' });
		expect(addGlossaryTerm(['Amélie'], 'AMÉLIE')).toEqual({ problem: 'duplicate' });
	});

	it('reports a blank entry, and adds nothing', () => {
		expect(addGlossaryTerm(['UNESCO'], '')).toEqual({ problem: 'blank' });
		expect(addGlossaryTerm(['UNESCO'], ' \t  ')).toEqual({ problem: 'blank' });
	});

	it('counts the length in characters, not UTF-16 units', () => {
		const longest = '𝔸'.repeat(MAX_GLOSSARY_TERM_LENGTH);
		expect(longest.length).toBe(MAX_GLOSSARY_TERM_LENGTH * 2);
		expect(addGlossaryTerm([], longest)).toMatchObject({ term: longest });
		expect(addGlossaryTerm([], `${longest}x`)).toEqual({ problem: 'length' });
		expect(addGlossaryTerm([], ` ${'a'.repeat(MAX_GLOSSARY_TERM_LENGTH)} `)).toMatchObject({
			term: 'a'.repeat(MAX_GLOSSARY_TERM_LENGTH)
		});
	});

	it('refuses a term already listed in any case', () => {
		expect(addGlossaryTerm(['Ki-Zerbo'], ' ki-zerbo ')).toEqual({ problem: 'duplicate' });
		expect(addGlossaryTerm(['Ki-Zerbo'], 'KI-ZERBO')).toEqual({ problem: 'duplicate' });
	});

	it('stops at the limit', () => {
		const full = terms(MAX_GLOSSARY_TERMS);
		expect(addGlossaryTerm(full, 'One more')).toEqual({ problem: 'full' });
		expect(addGlossaryTerm(full.slice(1), 'One more')).toMatchObject({ term: 'One more' });
	});
});

describe('removing a name or term', () => {
	it('removes it whatever its case, and leaves the others in order', () => {
		expect(removeGlossaryTerm(['UNESCO', 'Ki-Zerbo', 'Abidjan'], 'ki-zerbo')).toEqual([
			'UNESCO',
			'Abidjan'
		]);
	});

	it('leaves the list alone when the term is not in it', () => {
		expect(removeGlossaryTerm(['UNESCO'], 'Abidjan')).toEqual(['UNESCO']);
	});
});

describe('reading a stored list', () => {
	it('keeps a valid list as it is', () => {
		expect(normalizeGlossary(['UNESCO', 'Ki-Zerbo'])).toEqual(['UNESCO', 'Ki-Zerbo']);
	});

	it('reads anything that is not a list as no list', () => {
		for (const value of [undefined, null, 'UNESCO', 3, { 0: 'UNESCO' }])
			expect(normalizeGlossary(value)).toEqual([]);
	});

	it('drops what the core would refuse, keeping the first of each spelling', () => {
		expect(
			normalizeGlossary([
				' UNESCO ',
				42,
				null,
				'',
				'   ',
				'x'.repeat(MAX_GLOSSARY_TERM_LENGTH + 1),
				'unesco',
				['nested'],
				'Ki-Zerbo'
			])
		).toEqual(['UNESCO', 'Ki-Zerbo']);
	});

	it('keeps the first terms up to the limit', () => {
		const stored = terms(MAX_GLOSSARY_TERMS + 5);
		expect(normalizeGlossary(stored)).toEqual(stored.slice(0, MAX_GLOSSARY_TERMS));
	});
});
