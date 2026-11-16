import { beforeEach, expect, it } from 'vitest';
import { fireEvent, render, waitFor } from '@testing-library/svelte';
import TranscriptLog from './TranscriptLog.svelte';
import { setLocale } from './i18n';
import type { TranscriptLine } from './types';

/** Newest first, as the log store holds them; alternating sides so each is its own paragraph. */
function lines(count: number): TranscriptLine[] {
	return Array.from({ length: count }, (_, i) => ({
		id: i + 1,
		text: `Line ${i + 1}.`,
		sourceText: '',
		origin: i % 2 ? 'microphone' : 'system'
	})).reverse() as TranscriptLine[];
}

/** jsdom lays nothing out: give the log a scrollable geometry and a scroll position. */
function geometry(log: HTMLElement) {
	const box = { scrollHeight: 1000, clientHeight: 200, scrollTop: 0 };
	Object.defineProperty(log, 'scrollHeight', { configurable: true, get: () => box.scrollHeight });
	Object.defineProperty(log, 'clientHeight', { configurable: true, get: () => box.clientHeight });
	Object.defineProperty(log, 'scrollTop', {
		configurable: true,
		get: () => box.scrollTop,
		set: (value: number) => (box.scrollTop = value)
	});
	return box;
}

function mount(count = 4) {
	const view = render(TranscriptLog, {
		props: {
			mode: 'translate',
			transcript: lines(count),
			pending: [],
			reader: false,
			showOriginal: false
		}
	});
	const log = view.getByRole('region', { name: 'Transcript' });
	return { ...view, log, box: geometry(log) };
}

beforeEach(() => {
	setLocale('en');
	document.getSelection()?.removeAllRanges();
});

it('follows new text while the operator is at the bottom', async () => {
	const { box, rerender } = mount();
	box.scrollTop = 800;
	box.scrollHeight = 1200;
	await rerender({ transcript: lines(5) });
	expect(box.scrollTop).toBe(1200);
});

it('stops following while the operator has scrolled up, and offers Jump to latest', async () => {
	const { box, log, rerender, getByRole, queryByRole } = mount();
	expect(queryByRole('button', { name: 'Jump to latest' })).toBeNull();
	box.scrollTop = 300;
	await fireEvent.scroll(log);
	const jump = getByRole('button', { name: 'Jump to latest' });
	expect(jump).toHaveAttribute('aria-controls', log.id);

	box.scrollHeight = 1200;
	await rerender({ transcript: lines(5) });
	expect(box.scrollTop).toBe(300);

	await fireEvent.click(jump);
	expect(box.scrollTop).toBe(1200);
	expect(log).toHaveFocus();
	await waitFor(() => expect(queryByRole('button', { name: 'Jump to latest' })).toBeNull());
	// And it follows again from there.
	box.scrollHeight = 1400;
	await rerender({ transcript: lines(6) });
	expect(box.scrollTop).toBe(1400);
});

it('does not drag a selection the operator is making at the bottom', async () => {
	const { box, log, rerender, getByText } = mount();
	box.scrollTop = 800;
	document.getSelection()!.selectAllChildren(getByText('Line 2.'));
	box.scrollHeight = 1200;
	await rerender({ transcript: lines(5) });
	expect(box.scrollTop).toBe(800);
	expect(log.contains(document.getSelection()!.anchorNode)).toBe(true);

	document.getSelection()!.removeAllRanges();
	box.scrollHeight = 1300;
	await rerender({ transcript: lines(6) });
	expect(box.scrollTop).toBe(1300);
});

it('selects only the log on Ctrl+A, not the window around it', async () => {
	const outside = document.body.appendChild(document.createElement('p'));
	outside.textContent = 'Elsewhere in the window';
	try {
		const { log } = mount();
		const ctrlA = new KeyboardEvent('keydown', {
			key: 'a',
			ctrlKey: true,
			bubbles: true,
			cancelable: true
		});
		log.dispatchEvent(ctrlA);
		expect(ctrlA.defaultPrevented).toBe(true);
		const selected = document.getSelection()!.toString();
		expect(selected).toContain('Line 1.');
		expect(selected).toContain('Line 4.');
		expect(selected).not.toContain('Elsewhere');
		// A plain "a" is typing, not a command.
		const plain = new KeyboardEvent('keydown', { key: 'a', bubbles: true, cancelable: true });
		log.dispatchEvent(plain);
		expect(plain.defaultPrevented).toBe(false);
	} finally {
		outside.remove();
	}
});
