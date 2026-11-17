// One line of text typed by the operator, as the app keeps it and shows it: a marker's label,
// a notice on the overlay. Pure, so the overlay can check what it is handed with the same rule
// the operator window applied before sending it.

/** `value` as one line: control characters and line breaks become spaces, runs of space close
 *  up, and the result is cut to `max` characters — code points, so a cut never splits one.
 *  Anything that is not a string is the empty string. */
export function plainLine(value: unknown, max: number): string {
	if (typeof value !== 'string') return '';
	const flat = value
		.replace(/[\p{Cc}\u2028\u2029]+/gu, ' ')
		.replace(/\s+/g, ' ')
		.trim();
	return Array.from(flat).slice(0, Math.max(0, max)).join('').trim();
}
