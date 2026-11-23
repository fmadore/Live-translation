import { matchesSession } from './historySearch';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import { get } from 'svelte/store';
import { tick } from 'svelte';
import TranscriptHistory from './TranscriptHistory.svelte';
import { locale } from './i18n';
import { api } from './tauri';
import {
	historyEnabled,
	historyError,
	encodeSessionLog,
	historyRevision,
	sessionHistory,
	type HistoryListing,
	type SavedSession
} from './history';
import { transcript, clearTranscript } from './stores';

vi.mock('./tauri', () => ({
	isTauri: () => true,
	api: {
		listHistory: vi.fn(),
		deleteHistory: vi.fn(),
		writeHistory: vi.fn(),
		saveTranscript: vi.fn(),
		exportTranscripts: vi.fn()
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
/** What `list_history` answers for files the view does not hold yet. */
function listing(...files: { id: string; contents: string }[]): HistoryListing {
	return {
		sessions: files.map(({ id, contents }) => ({ id, length: contents.length, contents })),
		removed: []
	};
}
beforeEach(() => {
	locale.set('en');
	historyEnabled.set(false);
	historyError.set('');
	clearTranscript();
	vi.mocked(api.listHistory).mockResolvedValue(
		listing({ id: saved.id, contents: JSON.stringify(saved) })
	);
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
	vi.mocked(api.listHistory).mockResolvedValue({ sessions: [], removed: [saved.id] });
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
	vi.mocked(api.listHistory).mockResolvedValue(listing({ id: saved.id, contents: '{' }));
	const view = render(TranscriptHistory);
	await waitFor(() => expect(view.getByText('Unreadable session', { exact: false })).toBeTruthy());
	vi.mocked(api.listHistory).mockRejectedValue(new Error('Access denied'));
	await fireEvent.click(view.getByText('Refresh'));
	await waitFor(() => expect(view.getByRole('alert').textContent).toContain('Access denied'));
	view.unmount();
});

// The core rejects with an id and a detail, which `String()` turned into "[object Object]".
it('words what the core reports in the interface language, whichever way it arrives', async () => {
	const failure = { id: 'error.historyStorage', detail: 'C:\\History — Access is denied.' };
	vi.mocked(api.listHistory).mockRejectedValue(failure);
	const view = render(TranscriptHistory);
	try {
		// A failed listing, caught by the view itself.
		await waitFor(() =>
			expect(view.getByRole('alert').textContent).toBe(
				'Transcript history could not be updated. The history folder could not be read or changed (C:\\History — Access is denied.)'
			)
		);
		// A failed save, from the store the history writer reports to.
		historyError.set(failure);
		await waitFor(() => expect(view.getAllByRole('alert')).toHaveLength(2));
		// Kept structured, so both follow a change of interface language.
		locale.set('fr');
		await waitFor(() => {
			for (const alert of view.getAllByRole('alert'))
				expect(alert.textContent).toContain(
					'Le dossier de l’historique n’a pu être ni lu ni modifié (C:\\History — Access is denied.)'
				);
		});
		expect(view.container.textContent).not.toContain('[object Object]');
	} finally {
		view.unmount();
		locale.set('en');
	}
});

it('words a failed export from a saved session', async () => {
	vi.mocked(api.saveTranscript).mockRejectedValue({
		id: 'error.transcriptWrite',
		detail: 'D:\\Docs — Access is denied.'
	});
	const view = render(TranscriptHistory);
	await waitFor(() => expect(view.container.querySelector('.session')).not.toBeNull());
	await fireEvent.click(view.container.querySelector('.session')!);
	await fireEvent.click(view.getByText('Save as…'));
	await waitFor(() =>
		expect(view.getByRole('alert').textContent).toContain(
			'The transcript could not be written (D:\\Docs — Access is denied.)'
		)
	);
	view.unmount();
});

// Review U13: a failed copy, export, rename or delete was headed "Transcript history could not
// be updated.", which is what a failed listing says.
it('heads a failed action on a session apart from a failed listing', async () => {
	const headline = 'The last action on this session did not complete.';
	vi.mocked(api.saveTranscript).mockRejectedValue({
		id: 'error.transcriptWrite',
		detail: 'D:\\Docs — Access is denied.'
	});
	const writeText = vi.fn().mockRejectedValue(new Error('Clipboard blocked'));
	Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText } });
	const locked = { id: 'error.historyStorage', detail: 'C:\\History — in use' };
	const rename = vi.spyOn(sessionHistory, 'rename').mockRejectedValue(locked);
	const deletion = vi.spyOn(sessionHistory, 'delete').mockRejectedValue(locked);
	const view = render(TranscriptHistory);
	// As it reads: the markup's line breaks collapse on screen.
	const text = (element: HTMLElement) => element.textContent?.replace(/\s+/g, ' ').trim();
	const alert = () => text(view.getByRole('alert'));
	try {
		await waitFor(() => expect(view.container.querySelector('.session')).not.toBeNull());
		await fireEvent.click(view.container.querySelector('.session')!);

		await fireEvent.click(view.getByRole('button', { name: 'Save as…' }));
		await waitFor(() =>
			expect(alert()).toBe(
				`${headline} The transcript could not be written (D:\\Docs — Access is denied.)`
			)
		);
		await fireEvent.click(view.getByRole('button', { name: 'Copy transcript' }));
		await waitFor(() => expect(alert()).toBe(`${headline} Clipboard blocked`));
		await fireEvent.input(view.getByLabelText('Session title'), { target: { value: 'Workshop' } });
		await fireEvent.click(view.getByRole('button', { name: 'Save title' }));
		const storage = `${headline} The history folder could not be read or changed (C:\\History — in use)`;
		await waitFor(() => expect(rename).toHaveBeenCalled());
		await waitFor(() => expect(alert()).toBe(storage));
		await fireEvent.click(view.getByRole('button', { name: 'Delete' }));
		await fireEvent.click(view.getByRole('button', { name: 'Delete permanently' }));
		await waitFor(() => expect(deletion).toHaveBeenCalledWith(saved.id));
		await waitFor(() => expect(alert()).toBe(storage));

		// Worded where it is shown, so it follows a change of language.
		locale.set('fr');
		await waitFor(() =>
			expect(alert()).toContain('La dernière action sur cette session n’a pas abouti.')
		);
		locale.set('en');
		await waitFor(() => expect(alert()).toBe(storage));

		// A listing that fails keeps its own headline, beside the action's.
		vi.mocked(api.listHistory).mockRejectedValue(new Error('Access denied'));
		await fireEvent.click(view.getByRole('button', { name: 'Refresh' }));
		await waitFor(() => expect(view.getAllByRole('alert')).toHaveLength(2));
		expect(text(view.getAllByRole('alert')[0])).toBe(
			'Transcript history could not be updated. Access denied'
		);
		expect(text(view.getAllByRole('alert')[1])).toBe(storage);

		// The action's headline says "this session", so it goes with the selection.
		await fireEvent.click(view.container.querySelector('.session')!);
		expect(view.getAllByRole('alert')).toHaveLength(1);
		expect(alert()).not.toContain(headline);
	} finally {
		rename.mockRestore();
		deletion.mockRestore();
		view.unmount();
		locale.set('en');
	}
});

// Review U11: durations were written m:ss, so a two-hour session read "121:05".
it('writes a long session’s duration in hours', async () => {
	vi.mocked(api.listHistory).mockResolvedValue(
		listing({ id: saved.id, contents: JSON.stringify({ ...saved, durationMs: 7_265_000 }) })
	);
	const view = render(TranscriptHistory);
	await waitFor(() => expect(view.getByText(/^2:01:05 · /)).toBeInTheDocument());
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
	// Renaming a file from before the log format rewrites it, possibly at the same length, so
	// the renamed session is read again rather than claimed as held.
	await waitFor(() => expect(view.getByText('Session title saved.')).toBeTruthy());
	expect(vi.mocked(api.listHistory).mock.lastCall![0]).toEqual([]);
	rename.mockRestore();
	view.unmount();
});

it('allows explicit deletion of an unreadable record from the detail pane', async () => {
	vi.mocked(api.listHistory).mockResolvedValue(listing({ id: saved.id, contents: '{' }));
	const deletion = vi.spyOn(sessionHistory, 'delete').mockResolvedValue();
	const view = render(TranscriptHistory);
	await waitFor(() => expect(view.container.querySelector('.session')).not.toBeNull());
	await fireEvent.click(view.container.querySelector('.session')!);
	await fireEvent.click(view.getByText('Delete'));
	expect(deletion).not.toHaveBeenCalled();
	vi.mocked(api.listHistory).mockResolvedValue({ sessions: [], removed: [saved.id] });
	await fireEvent.click(view.getByText('Delete permanently'));
	await waitFor(() => expect(deletion).toHaveBeenCalledWith(saved.id));
	deletion.mockRestore();
	view.unmount();
});

// E6: every history write used to make an open tab read, send and decode every session again.
it('lists by what it already holds and keeps sessions the core says are unchanged', async () => {
	const contents = JSON.stringify(saved);
	const view = render(TranscriptHistory);
	await waitFor(() => expect(view.container.querySelector('.session')).not.toBeNull());
	expect(vi.mocked(api.listHistory).mock.lastCall![0]).toEqual([]);

	vi.mocked(api.listHistory).mockResolvedValue({
		sessions: [{ id: saved.id, length: contents.length }],
		removed: []
	});
	await fireEvent.click(view.getByText('Refresh'));
	await waitFor(() => expect(api.listHistory).toHaveBeenCalledTimes(2));
	expect(vi.mocked(api.listHistory).mock.lastCall![0]).toEqual([[saved.id, contents.length]]);
	await fireEvent.click(view.container.querySelector('.session')!);
	expect(view.getByText('Um, hello.')).toBeTruthy();

	vi.mocked(api.listHistory).mockResolvedValue({ sessions: [], removed: [saved.id] });
	await fireEvent.click(view.getByText('Refresh'));
	await waitFor(() => expect(view.getByText('No saved sessions yet.')).toBeTruthy());
	view.unmount();
});

// D18: a session captioned in two languages was listed, and filtered, by its first alone.
it('lists and filters a two-language session by both of its caption languages', async () => {
	// Written as a log: the second language arrived after the log format, so only logs have one.
	const both: SavedSession = { ...saved, secondTargetLanguage: 'de' };
	vi.mocked(api.listHistory).mockResolvedValue(
		listing({ id: both.id, contents: encodeSessionLog(both) })
	);
	const view = render(TranscriptHistory);
	await waitFor(() => expect(view.container.querySelector('.session')).not.toBeNull());
	expect(view.container.querySelector('.session')).toHaveTextContent(
		'Auto-detected speech → FR + DE'
	);
	const language = view.getByRole('combobox', { name: 'Caption language' });
	for (const [code, shown] of [
		['de', true],
		['fr', true],
		['en', false]
	] as const) {
		await fireEvent.change(language, { target: { value: code } });
		expect(view.container.querySelector('.session') !== null).toBe(shown);
	}
	view.unmount();
});

it('matches either caption language, and sessions saved before the second existed', () => {
	const filter = { query: '', from: '', to: '', language: 'de' };
	expect(matchesSession({ ...saved, secondTargetLanguage: 'de' }, filter)).toBe(true);
	expect(matchesSession({ ...saved, secondTargetLanguage: null }, filter)).toBe(false);
	// Older files carry no second language at all.
	expect(matchesSession(saved, { ...filter, language: 'fr' })).toBe(true);
	expect(matchesSession(saved, filter)).toBe(false);
	// Subtitles are filed under the language spoken.
	const subtitles: SavedSession = { ...saved, mode: 'transcribe', targetLanguage: null };
	expect(
		matchesSession({ ...subtitles, sourceLanguage: 'en' }, { ...filter, language: 'en' })
	).toBe(true);
	expect(matchesSession(subtitles, { ...filter, language: 'auto' })).toBe(true);
});

it('re-words a notice already on screen when the interface language changes', async () => {
	const rename = vi.spyOn(sessionHistory, 'rename').mockResolvedValue();
	const view = render(TranscriptHistory);
	await waitFor(() => expect(view.container.querySelector('.session')).not.toBeNull());
	await fireEvent.click(view.container.querySelector('.session')!);
	await fireEvent.input(view.getByLabelText('Session title'), { target: { value: 'Workshop' } });
	await fireEvent.click(view.getByText('Save title'));
	const notice = view.container.querySelector('.notice')!;
	await waitFor(() => expect(notice).toHaveTextContent('Session title saved.'));
	locale.set('fr');
	await tick();
	expect(notice).toHaveTextContent('Titre de la session enregistré.');
	rename.mockRestore();
	view.unmount();
});

it('words a failed rename from the catalogue, never as raw English', async () => {
	// What history.ts rejects with for a session deleted a moment before (review B18): the
	// core's error shape, or a message the window words itself.
	for (const failure of [
		{ id: 'error.whisperModel' },
		(m: { error: { whisperModel: string } }) => m.error.whisperModel
	]) {
		const rename = vi.spyOn(sessionHistory, 'rename').mockRejectedValue(failure);
		const view = render(TranscriptHistory);
		await waitFor(() => expect(view.container.querySelector('.session')).not.toBeNull());
		await fireEvent.click(view.container.querySelector('.session')!);
		await fireEvent.input(view.getByLabelText('Session title'), { target: { value: 'Gone' } });
		await fireEvent.click(view.getByText('Save title'));
		await waitFor(() =>
			expect(view.getByRole('alert')).toHaveTextContent('The Whisper model could not be prepared')
		);
		locale.set('fr');
		await tick();
		expect(view.getByRole('alert')).not.toHaveTextContent('The Whisper model');
		locale.set('en');
		rename.mockRestore();
		view.unmount();
	}
});

it('previews a two-language session with the original once, each line in its language', async () => {
	const both: SavedSession = {
		...saved,
		secondTargetLanguage: 'ar',
		lines: [
			{ ...saved.lines[0], lane: 1, language: 'ar', id: 2, text: 'مرحبا' },
			{ ...saved.lines[0], lane: 0, language: 'fr', id: 1, text: 'Euh, bonjour.' }
		]
	};
	both.lines[1].sourceText = 'Um, hello.';
	both.lines[0].sourceText = 'Um, hello.';
	vi.mocked(api.listHistory).mockResolvedValue(
		listing({ id: both.id, contents: encodeSessionLog(both) })
	);
	const view = render(TranscriptHistory);
	await waitFor(() => expect(view.container.querySelector('.session')).not.toBeNull());
	await fireEvent.click(view.container.querySelector('.session')!);
	const preview = view.getByRole('region', { name: 'Saved transcript' });
	const originals = preview.querySelectorAll('.source');
	expect(originals).toHaveLength(1);
	expect(originals[0]).toHaveAttribute('lang', '');
	expect(originals[0]).toHaveAttribute('dir', 'auto');
	const arabic = view.getByText('مرحبا');
	expect(arabic).toHaveAttribute('lang', 'ar');
	expect(arabic).toHaveAttribute('dir', 'rtl');
	expect(view.getByText('Euh, bonjour.')).toHaveAttribute('lang', 'fr');
	view.unmount();
});

it('words a rename refused because the session was just deleted (review B18)', async () => {
	const rename = vi
		.spyOn(sessionHistory, 'rename')
		.mockRejectedValue({ id: 'error.historySessionDeleted' });
	const view = render(TranscriptHistory);
	await waitFor(() => expect(view.container.querySelector('.session')).not.toBeNull());
	await fireEvent.click(view.container.querySelector('.session')!);
	await fireEvent.input(view.getByLabelText('Session title'), { target: { value: 'Gone' } });
	await fireEvent.click(view.getByText('Save title'));
	await waitFor(() =>
		expect(view.getByRole('alert')).toHaveTextContent('This session was deleted')
	);
	expect(view.getByRole('alert')).not.toHaveTextContent('Session deleted');
	rename.mockRestore();
	view.unmount();
});

it('shows only the opt-ins and one sentence until there is a saved session', async () => {
	vi.mocked(api.listHistory).mockResolvedValue(listing());
	const view = render(TranscriptHistory);
	await waitFor(() => expect(api.listHistory).toHaveBeenCalled());
	expect(view.getByText('No saved sessions yet.')).toBeTruthy();
	expect(view.queryByRole('button', { name: 'Refresh' })).toBeNull();
	expect(view.queryByRole('searchbox')).toBeNull();
	expect(view.queryByLabelText('Language')).toBeNull();
	expect(view.queryByText('Choose a session to read or export it.')).toBeNull();
	// The opt-in and the notice's live region are there from the start.
	expect(view.getAllByRole('checkbox').length).toBeGreaterThan(0);
	expect(view.getByRole('status')).toBeTruthy();

	// A listing that fails keeps Refresh, as the way to try again.
	vi.mocked(api.listHistory).mockRejectedValue(new Error('Access denied'));
	vi.useFakeTimers();
	try {
		historyRevision.update((n) => n + 1);
		await tick();
		await vi.advanceTimersByTimeAsync(5000);
	} finally {
		vi.useRealTimers();
	}
	await waitFor(() => expect(view.getByRole('alert')).toBeTruthy());
	const refresh = view.getByRole('button', { name: 'Refresh' });

	vi.mocked(api.listHistory).mockResolvedValue(
		listing({ id: saved.id, contents: JSON.stringify(saved) })
	);
	await fireEvent.click(refresh);
	await waitFor(() => expect(view.getByRole('searchbox')).toBeTruthy());
	expect(view.queryByText('No saved sessions yet.')).toBeNull();
	view.unmount();
});

// Several sessions at once (multi-select): a search narrows what select-all ticks, deletion
// asks once and counts, and saving writes every ticked session through one batch command.
describe('several sessions at once', () => {
	const keynote: SavedSession = { ...saved, title: 'Keynote' };
	const other: SavedSession = {
		...saved,
		id: '12345678-1234-1234-1234-123456789abd',
		title: 'Board meeting',
		startedAt: '2026-09-20T10:00:00Z',
		lines: [{ ...saved.lines[0], text: 'Quarterly figures.' }]
	};
	const untimed: SavedSession = {
		...saved,
		id: '12345678-1234-1234-1234-123456789abe',
		title: 'Recovered',
		lines: [{ id: 1, text: 'No timing here.', sourceText: '', origin: 'system' }]
	};
	beforeEach(() => {
		vi.mocked(api.listHistory).mockResolvedValue(
			listing(
				{ id: keynote.id, contents: JSON.stringify(keynote) },
				{ id: other.id, contents: JSON.stringify(other) },
				{ id: untimed.id, contents: JSON.stringify(untimed) }
			)
		);
	});
	const selectAll = (view: {
		getByRole: (role: 'checkbox', options: { name: string }) => HTMLElement;
	}) => view.getByRole('checkbox', { name: 'Select all shown' });

	it('selects what the search shows, and counts it', async () => {
		const view = render(TranscriptHistory);
		await waitFor(() => expect(view.container.querySelectorAll('.session')).toHaveLength(3));
		expect(view.getByText('None selected')).toBeTruthy();
		await fireEvent.input(view.getByLabelText('Search titles and transcript text'), {
			target: { value: 'board' }
		});
		await fireEvent.click(selectAll(view));
		expect(view.getByText('1 session selected')).toBeTruthy();
		expect(view.getByRole('checkbox', { name: 'Select Board meeting' })).toBeChecked();
		await fireEvent.click(view.getByText('Clear filters'));
		// The other two are shown again but were never ticked: select-all is part-way.
		expect(selectAll(view)).not.toBeChecked();
		expect((selectAll(view) as HTMLInputElement).indeterminate).toBe(true);
		await fireEvent.click(selectAll(view));
		expect(view.getByText('3 sessions selected')).toBeTruthy();
		await fireEvent.click(selectAll(view));
		expect(view.getByText('None selected')).toBeTruthy();
		view.unmount();
	});

	it('deletes the selection only after one confirmation that counts it', async () => {
		const deletion = vi
			.spyOn(sessionHistory, 'deleteMany')
			.mockResolvedValue({ deleted: [saved.id, other.id], failed: [] });
		const view = render(TranscriptHistory);
		await waitFor(() => expect(view.container.querySelectorAll('.session')).toHaveLength(3));
		await fireEvent.click(view.getByRole('checkbox', { name: 'Select Board meeting' }));
		await fireEvent.click(view.getByRole('checkbox', { name: 'Select Keynote' }));
		expect(view.getByText('2 sessions selected')).toBeTruthy();
		await fireEvent.click(view.getByText('Delete selected'));
		expect(deletion).not.toHaveBeenCalled();
		vi.mocked(api.listHistory).mockResolvedValue({
			sessions: [{ id: untimed.id, length: JSON.stringify(untimed).length }],
			removed: [saved.id, other.id]
		});
		await fireEvent.click(view.getByText('Delete 2 sessions permanently'));
		await waitFor(() => expect(view.getByText('2 sessions deleted.')).toBeTruthy());
		expect(deletion).toHaveBeenCalledWith(expect.arrayContaining([saved.id, other.id]));
		expect(view.container.querySelectorAll('.session')).toHaveLength(1);
		expect(view.getByText('None selected')).toBeTruthy();
		deletion.mockRestore();
		view.unmount();
	});

	// #235: armed while a filter hid some ticked sessions, the confirmation went on to delete
	// them all once the filter was cleared, more than it had counted.
	it('withdraws the batch confirmation when a filter changes what is selected', async () => {
		const deletion = vi.spyOn(sessionHistory, 'deleteMany');
		const view = render(TranscriptHistory);
		await waitFor(() => expect(view.container.querySelectorAll('.session')).toHaveLength(3));
		await fireEvent.click(view.getByRole('checkbox', { name: 'Select Board meeting' }));
		await fireEvent.click(view.getByRole('checkbox', { name: 'Select Keynote' }));
		const search = view.getByLabelText('Search titles and transcript text');
		await fireEvent.input(search, { target: { value: 'board' } });
		expect(view.getByText('1 session selected')).toBeTruthy();
		await fireEvent.click(view.getByText('Delete selected'));
		expect(view.getByText('Delete 1 session permanently')).toBeTruthy();
		await fireEvent.input(search, { target: { value: '' } });
		expect(view.getByText('2 sessions selected')).toBeTruthy();
		expect(view.queryByText('Delete 2 sessions permanently')).toBeNull();
		expect(view.getByText('Delete selected')).toBeTruthy();
		expect(deletion).not.toHaveBeenCalled();
		deletion.mockRestore();
		view.unmount();
	});

	it('reports the sessions a batch deletion could not delete', async () => {
		const deletion = vi.spyOn(sessionHistory, 'deleteMany').mockResolvedValue({
			deleted: [saved.id],
			failed: [
				{
					item: other.id,
					error: { id: 'error.historyStorage', detail: 'Access is denied.' }
				}
			]
		});
		const view = render(TranscriptHistory);
		await waitFor(() => expect(view.container.querySelectorAll('.session')).toHaveLength(3));
		await fireEvent.click(view.getByRole('checkbox', { name: 'Select Keynote' }));
		await fireEvent.click(view.getByRole('checkbox', { name: 'Select Board meeting' }));
		await fireEvent.click(view.getByText('Delete selected'));
		await fireEvent.click(view.getByText('Delete 2 sessions permanently'));
		const alert = await view.findByText(/1 of 2 selected sessions did not complete/);
		expect(alert.textContent).toContain('Access is denied.');
		// The one that failed stays ticked, so it can be tried again.
		expect(view.getByRole('checkbox', { name: 'Select Board meeting' })).toBeChecked();
		deletion.mockRestore();
		view.unmount();
	});

	it('saves every selected session in the chosen format and skips untimed ones for subtitles', async () => {
		vi.mocked(api.exportTranscripts).mockResolvedValue({
			directory: 'C:\Exports',
			written: ['a.srt', 'b.srt'],
			failed: []
		});
		const view = render(TranscriptHistory);
		await waitFor(() => expect(view.container.querySelectorAll('.session')).toHaveLength(3));
		await fireEvent.click(selectAll(view));
		await fireEvent.change(
			view.getByRole('combobox', { name: 'Export format for the selection' }),
			{ target: { value: 'srt' } }
		);
		await fireEvent.click(view.getByText('Save selected to a folder…'));
		await waitFor(() =>
			expect(
				view.getByText('2 transcripts saved to C:\Exports 1 session skipped', { exact: false })
			).toBeTruthy()
		);
		const files = vi.mocked(api.exportTranscripts).mock.calls[0][0];
		expect(files).toHaveLength(2);
		expect(files.every((file) => file.filename.endsWith('.srt'))).toBe(true);
		expect(files.map((file) => file.content).join(' ')).toContain('Quarterly figures.');
		view.unmount();
	});

	it('does nothing when the folder dialog is cancelled', async () => {
		vi.mocked(api.exportTranscripts).mockResolvedValue(null);
		const view = render(TranscriptHistory);
		await waitFor(() => expect(view.container.querySelectorAll('.session')).toHaveLength(3));
		await fireEvent.click(view.getByRole('checkbox', { name: 'Select Board meeting' }));
		await fireEvent.click(view.getByText('Save selected to a folder…'));
		await waitFor(() => expect(api.exportTranscripts).toHaveBeenCalledTimes(1));
		expect(view.container.querySelector('.notice')?.textContent).toBe('');
		expect(view.queryByRole('alert')).toBeNull();
		view.unmount();
	});
});
