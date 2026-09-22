import { matchesSession } from './historySearch';
import { beforeEach, expect, it, vi } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import { get } from 'svelte/store';
import { tick } from 'svelte';
import TranscriptHistory from './TranscriptHistory.svelte';
import { locale } from './i18n';
import { api } from './tauri';
import {
	historyEnabled,
	historyError,
	historyRevision,
	sessionHistory,
	type SavedSession
} from './history';
import { transcript, clearTranscript } from './stores';

vi.mock('./tauri', () => ({
	isTauri: () => true,
	api: {
		listHistory: vi.fn(),
		deleteHistory: vi.fn(),
		writeHistory: vi.fn(),
		saveTranscript: vi.fn()
	}
}));
const saved: SavedSession = {
	version: 1,
	id: '12345678-1234-1234-1234-123456789abc',
	startedAt: '2026-09-19T10:00:00Z',
	savedAt: '2026-09-19T10:01:00Z',
	endedAt: '2026-09-19T10:01:00Z',
	durationMs: 60000,
	mode: 'translate',
	sourceLanguage: 'auto',
	targetLanguage: 'fr',
	lines: [
		{
			id: 1,
			text: 'Um, hello.',
			sourceText: 'Euh, bonjour.',
			origin: 'system',
			startMs: 0,
			endMs: 1000
		}
	]
};
beforeEach(() => {
	locale.set('en');
	historyEnabled.set(false);
	historyError.set('');
	clearTranscript();
	vi.mocked(api.listHistory).mockResolvedValue([
		{ path: saved.id, contents: JSON.stringify(saved) }
	]);
	vi.mocked(api.saveTranscript).mockResolvedValue('saved.txt');
});

it('filters history with both German date fields and resets them', async () => {
	locale.set('de');
	const view = render(TranscriptHistory);
	try {
		await waitFor(() => expect(view.container.querySelector('.session')).not.toBeNull());
		const from = view.getByRole('textbox', { name: 'Ab Datum' });
		const to = view.getByRole('textbox', { name: 'Bis Datum' });
		expect(from).toHaveAttribute('placeholder', 'JJJJ-MM-TT');
		expect(to).toHaveAttribute('placeholder', 'JJJJ-MM-TT');
		await fireEvent.input(from, { target: { value: '2026-09-20' } });
		expect(view.container.querySelector('.session')).toBeNull();
		await fireEvent.input(from, { target: { value: '2026-09-19' } });
		await fireEvent.input(to, { target: { value: '2026-09-19' } });
		expect(view.container.querySelector('.session')).not.toBeNull();
		await fireEvent.input(to, { target: { value: '2026-09-18' } });
		expect(view.container.querySelector('.session')).toBeNull();
		await fireEvent.click(view.getByText('Filter zurücksetzen'));
		expect(from).toHaveValue('');
		expect(to).toHaveValue('');
		expect(view.container.querySelector('.session')).not.toBeNull();
	} finally {
		view.unmount();
		locale.set('en');
	}
});

it('reopens raw history without replacing the live transcript and exports every format', async () => {
	transcript.set([{ ...saved.lines[0], id: 99, text: 'Live session' }]);
	const view = render(TranscriptHistory);
	await waitFor(() => expect(view.container.querySelector('.session')).not.toBeNull());
	await fireEvent.click(view.container.querySelector('.session')!);
	expect(view.getByText('Um, hello.')).toBeTruthy();
	expect(view.getByText('Euh, bonjour.')).toBeTruthy();
	expect(get(transcript)[0].text).toBe('Live session');
	for (const format of ['text', 'markdown', 'srt', 'vtt']) {
		await fireEvent.change(view.getByRole('combobox', { name: 'Export format' }), {
			target: { value: format }
		});
		await fireEvent.click(view.getByText('Save as…'));
		await waitFor(() =>
			expect(api.saveTranscript).toHaveBeenCalledWith(
				expect.stringContaining('Um, hello.'),
				expect.any(String)
			)
		);
	}
	expect(api.saveTranscript).toHaveBeenCalledTimes(4);
	view.unmount();
});

it('copies and deletes a selected session only after confirmation', async () => {
	const writeText = vi.fn().mockResolvedValue(undefined);
	Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText } });
	const deletion = vi.spyOn(sessionHistory, 'delete').mockResolvedValue(undefined);
	const view = render(TranscriptHistory);
	await waitFor(() => expect(view.container.querySelector('.session')).not.toBeNull());
	await fireEvent.click(view.container.querySelector('.session')!);
	await fireEvent.click(view.getByText('Copy transcript'));
	await waitFor(() =>
		expect(writeText).toHaveBeenCalledWith(expect.stringContaining('Um, hello.'))
	);
	await fireEvent.click(view.getByText('Delete'));
	expect(deletion).not.toHaveBeenCalled();
	vi.mocked(api.listHistory).mockResolvedValue([]);
	await fireEvent.click(view.getByText('Delete permanently'));
	await waitFor(() => expect(view.getByText('No saved sessions yet.')).toBeTruthy());
	expect(deletion).toHaveBeenCalledWith(saved.id);
	deletion.mockRestore();
	view.unmount();
});

// Every finalized line of a recording session lands as a history write. The open tab used to
// re-list the whole folder for each one; it now catches up at most every five seconds.
it('re-lists history at most every few seconds while writes keep landing', async () => {
	const view = render(TranscriptHistory);
	await waitFor(() => expect(api.listHistory).toHaveBeenCalledTimes(1));
	vi.useFakeTimers();
	try {
		for (let write = 0; write < 5; write++) {
			historyRevision.update((n) => n + 1);
			await tick();
		}
		expect(api.listHistory).toHaveBeenCalledTimes(1);
		await vi.advanceTimersByTimeAsync(5000);
		expect(api.listHistory).toHaveBeenCalledTimes(2);
	} finally {
		vi.useRealTimers();
		view.unmount();
	}
});

it('shows unreadable records and read errors instead of silently losing history', async () => {
	vi.mocked(api.listHistory).mockResolvedValue([{ path: saved.id, contents: '{' }]);
	const view = render(TranscriptHistory);
	await waitFor(() => expect(view.getByText('Unreadable session', { exact: false })).toBeTruthy());
	vi.mocked(api.listHistory).mockRejectedValue(new Error('Access denied'));
	await fireEvent.click(view.getByText('Refresh'));
	await waitFor(() => expect(view.getByRole('alert').textContent).toContain('Access denied'));
	view.unmount();
});

it('searches titles, original source and translated text with inclusive local dates', () => {
	const filter = { query: 'BONJOUR', from: '', to: '', language: 'fr' };
	expect(matchesSession(saved, filter)).toBe(true);
	expect(matchesSession({ ...saved, title: 'Workshop' }, { ...filter, query: 'workshop' })).toBe(
		true
	);
	expect(matchesSession(saved, { ...filter, language: 'en' })).toBe(false);
	expect(matchesSession(saved, { ...filter, from: '2099-01-01' })).toBe(false);
	const localDate = new Date(saved.startedAt);
	const day = [
		localDate.getFullYear(),
		String(localDate.getMonth() + 1).padStart(2, '0'),
		String(localDate.getDate()).padStart(2, '0')
	].join('-');
	expect(matchesSession(saved, { ...filter, from: day, to: day })).toBe(true);
});
it('filters the session list and saves a title without changing raw text', async () => {
	const rename = vi.spyOn(sessionHistory, 'rename').mockResolvedValue();
	const view = render(TranscriptHistory);
	await waitFor(() => expect(view.container.querySelector('.session')).not.toBeNull());
	await fireEvent.input(view.getByLabelText('Search titles and transcript text'), {
		target: { value: 'missing' }
	});
	expect(view.getByText('No matching sessions.')).toBeTruthy();
	await fireEvent.click(view.getByText('Clear filters'));
	expect(view.getByLabelText('Search titles and transcript text')).toHaveValue('');
	expect(view.container.querySelector('.session')).not.toBeNull();
	await fireEvent.input(view.getByLabelText('Search titles and transcript text'), {
		target: { value: 'bonjour' }
	});
	await fireEvent.click(view.container.querySelector('.session')!);
	await fireEvent.input(view.getByLabelText('Session title'), { target: { value: 'Workshop' } });
	await fireEvent.click(view.getByText('Save title'));
	await waitFor(() =>
		expect(rename).toHaveBeenCalledWith(expect.objectContaining({ lines: saved.lines }), 'Workshop')
	);
	rename.mockRestore();
	view.unmount();
});

it('allows explicit deletion of an unreadable record from the detail pane', async () => {
	vi.mocked(api.listHistory).mockResolvedValue([{ path: saved.id, contents: '{' }]);
	const deletion = vi.spyOn(sessionHistory, 'delete').mockResolvedValue();
	const view = render(TranscriptHistory);
	await waitFor(() => expect(view.container.querySelector('.session')).not.toBeNull());
	await fireEvent.click(view.container.querySelector('.session')!);
	await fireEvent.click(view.getByText('Delete'));
	expect(deletion).not.toHaveBeenCalled();
	vi.mocked(api.listHistory).mockResolvedValue([]);
	await fireEvent.click(view.getByText('Delete permanently'));
	await waitFor(() => expect(deletion).toHaveBeenCalledWith(saved.id));
	deletion.mockRestore();
	view.unmount();
});
