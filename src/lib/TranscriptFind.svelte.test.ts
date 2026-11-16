// Find in the transcript reader, and markers in the running transcript, as the operator uses
// them: by role and accessible name, from the keyboard.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import { fireEvent, render, screen, waitFor, within } from '@testing-library/svelte';

vi.mock('./tauri', () => ({
	isTauri: () => true,
	api: { saveTranscript: vi.fn(), clearRecovery: vi.fn() }
}));

const TranscriptMonitor = (await import('./TranscriptMonitor.svelte')).default;
const { clearTranscript, markers, restoreTranscript, sessionStartedAt, transcriptDirty } =
	await import('./stores');
const { setLocale } = await import('./i18n');
import type { TranscriptLine } from './types';

const LINES: TranscriptLine[] = [
	{
		id: 3,
		text: 'The budget was approved.',
		sourceText: '',
		origin: 'system',
		startMs: 20_000,
		endMs: 21_000
	},
	{
		id: 2,
		text: 'Questions about the café?',
		sourceText: '',
		origin: 'microphone',
		startMs: 9_000,
		endMs: 10_000
	},
	{
		id: 1,
		text: 'Budget first, then the Café plan.',
		sourceText: '',
		origin: 'system',
		startMs: 0,
		endMs: 1_000
	}
];

function mount(
	props: { reader?: boolean; live?: boolean; findRequest?: number; markerRequest?: number } = {}
) {
	return render(TranscriptMonitor, {
		props: {
			mode: 'transcribe' as const,
			transcript: LINES,
			onError: vi.fn(),
			live: true,
			reader: true,
			...props
		}
	});
}

/** A registry standing in for `CSS.highlights`, which jsdom does not have. */
class FakeHighlight {
	ranges: Range[];
	constructor(...ranges: Range[]) {
		this.ranges = ranges;
	}
}
let registry: Map<string, FakeHighlight>;

beforeEach(() => {
	setLocale('en');
	clearTranscript();
	restoreTranscript(LINES);
	registry = new Map();
	vi.stubGlobal('Highlight', FakeHighlight);
	vi.stubGlobal('CSS', { highlights: registry });
	// jsdom lays nothing out, and has no box for a range at all.
	Range.prototype.getBoundingClientRect = () => new DOMRect();
});

afterEach(() => {
	vi.useRealTimers();
	vi.unstubAllGlobals();
});

describe('find in the transcript reader', () => {
	it('counts the matches as History does, steps with Enter and Shift+Enter, and marks them', async () => {
		mount();
		const open = screen.getByRole('button', { name: 'Find' });
		open.focus();
		await fireEvent.click(open);
		const field = screen.getByRole('searchbox', { name: 'Find in the transcript' });
		await waitFor(() => expect(document.activeElement).toBe(field));
		await fireEvent.input(field, { target: { value: 'BUDGET' } });
		// Searched after a pause in typing, and said by the status region.
		const status = within(screen.getByRole('search')).getByRole('status');
		await waitFor(() => expect(status).toHaveTextContent('1 of 2'));
		await waitFor(() => expect(registry.get('transcript-find')?.ranges).toHaveLength(2));
		expect(registry.get('transcript-find-current')?.ranges[0].toString()).toBe('Budget');
		await fireEvent.keyDown(field, { key: 'Enter' });
		await waitFor(() => expect(status).toHaveTextContent('2 of 2'));
		expect(registry.get('transcript-find-current')?.ranges[0].toString()).toBe('budget');
		await fireEvent.keyDown(field, { key: 'Enter', shiftKey: true });
		await waitFor(() => expect(status).toHaveTextContent('1 of 2'));
		// An accent typed apart is the same letter.
		await fireEvent.input(field, { target: { value: 'café' } });
		await waitFor(() => expect(status).toHaveTextContent('1 of 2'));
		await fireEvent.input(field, { target: { value: 'nothing like it' } });
		await waitFor(() => expect(status).toHaveTextContent('No matches'));
		expect(screen.getByRole('button', { name: 'Next match' })).toBeDisabled();
	});

	it('closes on Escape, unmarks the text and gives focus back', async () => {
		mount();
		const open = screen.getByRole('button', { name: 'Find' });
		open.focus();
		await fireEvent.click(open);
		const field = screen.getByRole('searchbox', { name: 'Find in the transcript' });
		await fireEvent.input(field, { target: { value: 'budget' } });
		await waitFor(() => expect(registry.has('transcript-find')).toBe(true));
		await fireEvent.keyDown(field, { key: 'Escape' });
		expect(screen.queryByRole('search')).toBeNull();
		expect(registry.size).toBe(0);
		expect(document.activeElement).toBe(open);
	});

	it('opens from the window’s shortcut only in the reader, and leaves the copy text alone', async () => {
		const view = mount({ reader: false });
		expect(screen.queryByRole('button', { name: 'Find' })).toBeNull();
		await view.rerender({ findRequest: 1 });
		expect(screen.queryByRole('search')).toBeNull();
		await view.rerender({ reader: true, findRequest: 2 });
		const field = await screen.findByRole('searchbox', { name: 'Find in the transcript' });
		await fireEvent.input(field, { target: { value: 'budget' } });
		await waitFor(() => expect(registry.has('transcript-find')).toBe(true));
		// Highlighting adds no element: the log's text is exactly what it was.
		const log = screen.getByRole('region', { name: 'Transcript' });
		expect(log.querySelector('mark')).toBeNull();
		expect(log.textContent).toContain('Budget first, then the Café plan.');
		// Leaving the reader closes it.
		await view.rerender({ reader: false });
		expect(screen.queryByRole('search')).toBeNull();
	});
});

describe('markers in the running transcript', () => {
	it('adds one with its text at the session’s time, as a heading in the log, and says so', async () => {
		sessionStartedAt.set(Date.now() - 125_000);
		mount({ reader: false });
		const add = screen.getByRole('button', { name: 'Add marker' });
		add.focus();
		await fireEvent.click(add);
		const field = screen.getByRole('textbox', { name: 'Marker text (optional)' });
		await waitFor(() => expect(document.activeElement).toBe(field));
		await fireEvent.input(field, { target: { value: 'Talk 2 starts' } });
		await fireEvent.submit(field.closest('form')!);
		const [marker] = get(markers);
		expect(marker.label).toBe('Talk 2 starts');
		expect(marker.atMs).toBeGreaterThanOrEqual(125_000);
		expect(get(transcriptDirty)).toBe(true);
		expect(screen.getByRole('heading', { name: 'Talk 2 starts' })).toBeInTheDocument();
		expect(
			screen
				.getAllByRole('status')
				.some((s) => /Marker added at 0:02:0\d: Talk 2 starts/.test(s.textContent ?? ''))
		).toBe(true);
		await waitFor(() => expect(document.activeElement).toBe(add));
		sessionStartedAt.set(null);
	});

	it('numbers a blank one, opens from the shortcut, and puts the field away on Escape', async () => {
		const view = mount({ reader: false });
		await view.rerender({ markerRequest: 1 });
		const field = await screen.findByRole('textbox', { name: 'Marker text (optional)' });
		await fireEvent.keyDown(field, { key: 'Escape' });
		expect(screen.queryByRole('textbox', { name: 'Marker text (optional)' })).toBeNull();
		expect(get(markers)).toHaveLength(0);
		await view.rerender({ markerRequest: 2 });
		await fireEvent.submit(
			(await screen.findByRole('textbox', { name: 'Marker text (optional)' })).closest('form')!
		);
		expect(screen.getByRole('heading', { name: 'Section 1' })).toBeInTheDocument();
	});

	it('is offered only while a session runs', () => {
		mount({ live: false, reader: false });
		expect(screen.queryByRole('button', { name: 'Add marker' })).toBeNull();
	});
});
