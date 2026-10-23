import { beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import { fireEvent, render, waitFor, within } from '@testing-library/svelte';

const mocks = vi.hoisted(() => ({
	saveTranscript: vi.fn(),
	clearRecovery: vi.fn()
}));

vi.mock('./tauri', () => ({
	isTauri: () => true,
	api: { saveTranscript: mocks.saveTranscript, clearRecovery: mocks.clearRecovery }
}));

const TranscriptMonitor = (await import('./TranscriptMonitor.svelte')).default;
const {
	clearTranscript,
	exportOriginal,
	pushCaption,
	recoveryEnabled,
	restoreTranscript,
	transcript,
	transcriptDirty
} = await import('./stores');
const { TRANSCRIPT_WARN_LINES } = await import('./document');
const { exportFormat, EXPORT_FORMAT_KEY } = await import('./exportFormat');
import type { TranscriptLine } from './types';

/** Seed the log the way a recovered or already-running session leaves it: lines present,
 *  newest first, none of them on disk. */
function seed(count: number): TranscriptLine[] {
	const lines: TranscriptLine[] = Array.from({ length: count }, (_, i) => ({
		id: count - i,
		text: `utterance ${count - i}`,
		sourceText: `source ${count - i}`,
		origin: 'microphone'
	}));
	restoreTranscript(lines);
	return lines;
}

function mount(lines: TranscriptLine[], extra: { live?: boolean; reader?: boolean } = {}) {
	const onError = vi.fn();
	const view = render(TranscriptMonitor, {
		props: { mode: 'transcribe' as const, transcript: lines, onError, ...extra }
	});
	return { ...view, onError };
}

beforeEach(() => {
	clearTranscript();
	recoveryEnabled.set(false);
	// A remembered preference now, so one test's choice would otherwise carry into the next.
	exportFormat.set('markdown');
	mocks.saveTranscript.mockResolvedValue('C:\\Docs\\Live-translation\\transcript.md');
	mocks.clearRecovery.mockResolvedValue(undefined);
});

describe('saved / unsaved state', () => {
	it('explains why old untimed recovery cannot be exported as subtitles', async () => {
		const { getByRole, getByText } = mount(seed(2));
		await fireEvent.change(getByRole('combobox', { name: 'Export format' }), {
			target: { value: 'srt' }
		});
		expect(getByRole('button', { name: 'Save as…' })).toBeDisabled();
		expect(getByText(/recovered transcript has no timing/)).toBeInTheDocument();
		await fireEvent.change(getByRole('combobox', { name: 'Export format' }), {
			target: { value: 'text' }
		});
		expect(getByRole('button', { name: 'Save as…' })).toBeEnabled();
	});
	it('exports the selected timed format through Save As', async () => {
		const lines = seed(1).map((line) => ({ ...line, startMs: 0, endMs: 1000 }));
		restoreTranscript(lines);
		const { getByRole } = mount(lines);
		await fireEvent.change(getByRole('combobox', { name: 'Export format' }), {
			target: { value: 'vtt' }
		});
		await fireEvent.click(getByRole('button', { name: 'Save as…' }));
		await waitFor(() => expect(mocks.saveTranscript).toHaveBeenCalled());
		const [content, filename] = mocks.saveTranscript.mock.lastCall!;
		expect(content).toMatch(/^WEBVTT/);
		expect(filename).toMatch(/\.vtt$/);
	});
	// The page mounts the monitor once while a session runs and again after it, and each
	// mount used to start from Markdown.
	it('remembers the chosen format when it is mounted again', async () => {
		const lines = seed(1).map((line) => ({ ...line, startMs: 0, endMs: 1000 }));
		restoreTranscript(lines);
		const first = mount(lines);
		await fireEvent.change(first.getByRole('combobox', { name: 'Export format' }), {
			target: { value: 'srt' }
		});
		expect(localStorage.getItem(EXPORT_FORMAT_KEY)).toBe('srt');
		first.unmount();
		const second = mount(lines);
		expect(second.getByRole('combobox', { name: 'Export format' })).toHaveValue('srt');
		await fireEvent.click(second.getByRole('button', { name: 'Save as…' }));
		await waitFor(() => expect(mocks.saveTranscript).toHaveBeenCalled());
		expect(mocks.saveTranscript.mock.lastCall![1]).toMatch(/\.srt$/);
	});
	it('withdraws the clear confirmation when new lines arrive', async () => {
		const { getByRole, rerender } = mount(seed(2));
		await fireEvent.click(getByRole('button', { name: 'Clear' }));
		expect(getByRole('button', { name: 'Discard unsaved lines?' })).toBeInTheDocument();
		await rerender({ mode: 'transcribe', transcript: seed(3), onError: vi.fn() });
		expect(getByRole('button', { name: 'Clear' })).toBeInTheDocument();
		expect(get(transcript)).toHaveLength(3);
	});
	// Issue #25: on screen a transcript that exists only in memory looked exactly like one
	// that was already on disk.
	it('marks a fresh log as unsaved', () => {
		const { getByTestId } = mount(seed(3));
		expect(getByTestId('save-state')).toHaveTextContent('Unsaved');
	});

	it('says nothing about save state when there is no transcript', () => {
		const { queryByTestId } = mount([]);
		expect(queryByTestId('save-state')).toBeNull();
	});

	it('flips to saved and names the file once the write lands', async () => {
		const { getByTestId, getByText } = mount(seed(3));

		await fireEvent.click(getByText('Save as…'));

		await waitFor(() => expect(getByTestId('save-state')).toHaveTextContent('Saved'));
		expect(getByText('C:\\Docs\\Live-translation\\transcript.md')).toBeInTheDocument();
	});

	it('stays saved when the operator saves the same document again', async () => {
		const { getByTestId, getByText } = mount(seed(3));

		await fireEvent.click(getByText('Save as…'));
		await waitFor(() => expect(getByTestId('save-state')).toHaveTextContent('Saved'));
		await fireEvent.click(getByText('Save as…'));

		await waitFor(() => expect(mocks.saveTranscript).toHaveBeenCalledTimes(2));
		expect(getByTestId('save-state')).toHaveTextContent('Saved');
		expect(get(transcriptDirty)).toBe(false);
	});

	it('reports a failed write and stays unsaved', async () => {
		mocks.saveTranscript.mockRejectedValue(new Error('could not create "D:\\Docs"'));
		const { getByTestId, getByText, onError } = mount(seed(2));

		await fireEvent.click(getByText('Save as…'));

		await waitFor(() =>
			expect(onError).toHaveBeenCalledWith(expect.stringContaining('could not create'))
		);
		expect(getByTestId('save-state')).toHaveTextContent('Unsaved');
	});
});

describe('discarding', () => {
	// Clear is a data-loss button. With unsaved text behind it, one stray click must not be
	// the whole interaction.
	it('asks once before discarding unsaved lines', async () => {
		const { getByText } = mount(seed(2));

		await fireEvent.click(getByText('Clear'));

		expect(getByText('Discard unsaved lines?')).toBeInTheDocument();
		expect(get(transcript)).toHaveLength(2);

		await fireEvent.click(getByText('Discard unsaved lines?'));
		expect(get(transcript)).toHaveLength(0);
	});

	it('retires the recovery spool along with the text it covered', async () => {
		const { getByText } = mount(seed(2));

		await fireEvent.click(getByText('Clear'));
		await fireEvent.click(getByText('Discard unsaved lines?'));

		await waitFor(() => expect(mocks.clearRecovery).toHaveBeenCalled());
	});

	it('clears a saved log without asking', async () => {
		const { getByText } = mount(seed(2));

		await fireEvent.click(getByText('Save as…'));
		await waitFor(() => expect(get(transcriptDirty)).toBe(false));
		await fireEvent.click(getByText('Clear'));

		expect(get(transcript)).toHaveLength(0);
	});
});

describe('long sessions', () => {
	it('preserves reading position on new text and follows again after Jump to latest', async () => {
		const { getByRole, rerender } = mount(seed(10));
		const log = getByRole('region', { name: 'Transcript' });
		Object.defineProperties(log, {
			scrollHeight: { configurable: true, value: 1000 },
			clientHeight: { configurable: true, value: 180 }
		});
		log.scrollTop = 120;
		await fireEvent.scroll(log);
		await rerender({ mode: 'transcribe', transcript: seed(11), onError: vi.fn() });
		expect(log.scrollTop).toBe(120);
		await fireEvent.click(getByRole('button', { name: 'Jump to latest' }));
		expect(log.scrollTop).toBe(1000);
		expect(log).toHaveFocus();
		Object.defineProperty(log, 'scrollHeight', { value: 1200 });
		await rerender({ mode: 'transcribe', transcript: seed(12), onError: vi.fn() });
		expect(log.scrollTop).toBe(1200);
	});

	// The warning is one of several `role="status"` regions in the component now, so these
	// address it by its text and assert separately that it is a region at all.
	const WARNING = /Nothing is being dropped/;

	it('keeps a log well past the old 1,000-line cap and says it should be saved', () => {
		const lines = seed(TRANSCRIPT_WARN_LINES);
		const { getByText } = mount(lines);

		expect(get(transcript)).toHaveLength(TRANSCRIPT_WARN_LINES);
		expect(getByText(WARNING)).toHaveAttribute('role', 'status');
	});

	it('does not nag about a long session that is already on disk', async () => {
		const { getByText, queryByText } = mount(seed(TRANSCRIPT_WARN_LINES));

		await fireEvent.click(getByText('Save as…'));

		await waitFor(() => expect(queryByText(WARNING)).toBeNull());
	});

	it('stays quiet below the threshold', () => {
		const { queryByText } = mount(seed(TRANSCRIPT_WARN_LINES - 1));
		expect(queryByText(WARNING)).toBeNull();
	});
});

// Issue #24: a save that only turns a badge from "Unsaved" to "Saved" is invisible to a
// screen reader, and the operator saving a transcript is exactly the moment that needs a
// confirmation they can hear.
describe('announcing a save', () => {
	it('reports the path a screen reader would otherwise never hear', async () => {
		const { getByText, container } = mount(seed(3));

		await fireEvent.click(getByText('Save as…'));

		await waitFor(() => {
			const region = container.querySelector('p.sr-only[role="status"]');
			expect(region).toHaveTextContent(/Transcript saved to .*transcript\.md/);
		});
	});

	it('is silent until something has been saved', () => {
		const { container } = mount(seed(3));
		expect(container.querySelector('p.sr-only[role="status"]')).toHaveTextContent('');
	});
});

describe('the recovery opt-in', () => {
	it('is off unless the operator asks for it', () => {
		const { getByRole } = mount(seed(1));
		expect(getByRole('checkbox', { name: /recovery copy/ })).not.toBeChecked();
	});

	it('turns the spool on without touching disk itself', async () => {
		const { getByRole } = mount(seed(1));

		await fireEvent.click(getByRole('checkbox', { name: /recovery copy/ }));

		expect(get(recoveryEnabled)).toBe(true);
		expect(mocks.clearRecovery).not.toHaveBeenCalled();
	});

	// "Disabled" has to mean there is nothing left on disk, not just that nothing more is
	// being written.
	it('deletes what it already wrote when switched back off', async () => {
		recoveryEnabled.set(true);
		const { getByRole } = mount(seed(1));

		await fireEvent.click(getByRole('checkbox', { name: /recovery copy/ }));

		expect(get(recoveryEnabled)).toBe(false);
		await waitFor(() => expect(mocks.clearRecovery).toHaveBeenCalledTimes(1));
	});
});

describe('the bilingual export option', () => {
	it('is offered only when the lines carry original speech, and remembers the choice', async () => {
		exportOriginal.set(false);
		const { getByRole } = mount(seed(2));
		const option = getByRole('checkbox', { name: 'Include original speech' });
		expect(option).not.toBeChecked();
		await fireEvent.click(option);
		expect(get(exportOriginal)).toBe(true);
		exportOriginal.set(false);
	});

	it('is not offered for subtitles, which have no separate original', () => {
		const lines = seed(2).map((line) => ({ ...line, sourceText: '' }));
		const { queryByRole } = mount(lines);
		expect(queryByRole('checkbox', { name: 'Include original speech' })).toBeNull();
	});
});

// Issue #124: the operator reads back during the meeting and hands the last few minutes to an
// assistant, without stopping the session.
describe('copying', () => {
	const writeText = vi.fn();
	beforeEach(() => {
		writeText.mockReset().mockResolvedValue(undefined);
		Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText } });
	});

	function timed(): TranscriptLine[] {
		const lines: TranscriptLine[] = [
			{
				id: 2,
				text: 'the late remark',
				sourceText: '',
				origin: 'system',
				startMs: 400_000,
				endMs: 410_000
			},
			{
				id: 1,
				text: 'the early remark',
				sourceText: '',
				origin: 'system',
				startMs: 0,
				endMs: 60_000
			}
		];
		restoreTranscript(lines);
		return lines;
	}

	it('takes the last minutes of a timed log', async () => {
		const { getByRole, getByText } = mount(timed());
		await fireEvent.change(getByRole('combobox', { name: 'What to copy' }), {
			target: { value: '5' }
		});
		await fireEvent.click(getByRole('button', { name: 'Copy' }));

		await waitFor(() => expect(writeText).toHaveBeenCalled());
		expect(writeText.mock.lastCall![0]).toContain('the late remark');
		expect(writeText.mock.lastCall![0]).not.toContain('the early remark');
		expect(getByText('Copied the last 5 minutes.', { selector: '[aria-hidden]' })).toBeVisible();
	});

	it('copies a log without timing whole, and says so in the range it shows', async () => {
		const { getByRole } = mount(seed(3));
		const range = getByRole('combobox', { name: 'What to copy' });
		expect(range).toBeDisabled();
		expect(range).toHaveValue('0');

		await fireEvent.click(getByRole('button', { name: 'Copy' }));

		await waitFor(() => expect(writeText).toHaveBeenCalled());
		for (const n of [1, 2, 3]) expect(writeText.mock.lastCall![0]).toContain(`utterance ${n}`);
	});

	it('includes the turn still being spoken while a session runs', async () => {
		const lines = timed();
		pushCaption({
			turnId: 9,
			text: 'a sentence still going',
			sourceText: '',
			final: false,
			origin: 'microphone',
			startMs: 405_000,
			endMs: 420_000
		});
		const { getByRole } = mount(lines, { live: true });
		await fireEvent.click(getByRole('button', { name: 'Copy' }));

		await waitFor(() => expect(writeText).toHaveBeenCalled());
		const text = writeText.mock.lastCall![0] as string;
		expect(text.indexOf('a sentence still going')).toBeGreaterThan(text.indexOf('the late remark'));
	});

	it('reports a refused clipboard', async () => {
		writeText.mockRejectedValue(new Error('clipboard refused'));
		const { getByRole, onError } = mount(seed(1));
		await fireEvent.click(getByRole('button', { name: 'Copy' }));
		await waitFor(() =>
			expect(onError).toHaveBeenCalledWith(expect.stringContaining('clipboard refused'))
		);
	});
});

// Review U3: a spool or history write that keeps failing is said beside the save badge for as
// long as it lasts, not only once on a status line the next status overwrites.
describe('background saving that fails', () => {
	it('says so beside the save badge while it lasts, with the failure as its tooltip', async () => {
		const { recoveryFailure } = await import('./recovery');
		const { historyEnabled, historyError } = await import('./history');
		const { getByText, queryByText } = mount(seed(2));
		expect(queryByText('Recovery copy failing')).toBeNull();

		recoveryEnabled.set(true);
		recoveryFailure.set({ id: 'error.recoveryWrite', detail: 'disk full' });
		const spool = await waitFor(() => getByText('Recovery copy failing'));
		expect(spool).toHaveAttribute('title', 'The recovery copy could not be written (disk full)');

		historyEnabled.set(true);
		historyError.set('access denied');
		await waitFor(() =>
			expect(getByText('History not saving', { selector: 'span' })).toBeInTheDocument()
		);
		// The spool's first failure reaches the status line; history's is said here, once.
		expect(
			within(document.body)
				.getAllByRole('status')
				.map((region) => region.textContent)
		).toContain('History not saving');

		recoveryFailure.set('');
		historyError.set('');
		await waitFor(() => expect(queryByText('Recovery copy failing')).toBeNull());
		expect(queryByText('History not saving', { selector: 'span' })).toBeNull();
		// Switched off, a failure from before says nothing.
		recoveryEnabled.set(false);
		recoveryFailure.set('stale');
		historyEnabled.set(false);
		await waitFor(() => expect(queryByText('Recovery copy failing')).toBeNull());
		recoveryFailure.set('');
	});
});

describe('the transcript reader', () => {
	const interim = {
		turnId: 4,
		text: 'and the budget for next year',
		sourceText: 'et le budget de l’an prochain',
		final: false,
		origin: 'system' as const,
		startMs: 1000,
		endMs: 2000
	};

	it('shows the turn still being spoken at the foot of the log', () => {
		pushCaption(interim);
		const { getByRole } = mount(seed(2), { live: true, reader: true });
		const rows = within(getByRole('region', { name: 'Transcript' })).getAllByRole('listitem');
		expect(rows.at(-1)).toHaveTextContent(interim.text);
		expect(rows.at(-1)).toHaveAccessibleDescription('Still being spoken');
	});

	it('leaves the turn being spoken to the live view', () => {
		pushCaption(interim);
		const { queryByText } = mount(seed(2), { live: true });
		expect(queryByText(interim.text)).toBeNull();
	});

	it('shows the original speech exactly when it would be exported', async () => {
		exportOriginal.set(false);
		const { queryByText } = mount(seed(2), { reader: true });
		expect(queryByText(/source 2/, { selector: '.original' })).toBeNull();
		exportOriginal.set(true);
		await waitFor(() =>
			expect(queryByText(/source 2/, { selector: '.original' })).toBeInTheDocument()
		);
		exportOriginal.set(false);
	});

	it('holds still while the operator is selecting text at the bottom', async () => {
		const { getByRole, rerender } = mount(seed(10), { reader: true });
		const log = getByRole('region', { name: 'Transcript' });
		Object.defineProperties(log, {
			scrollHeight: { configurable: true, value: 1000 },
			clientHeight: { configurable: true, value: 1000 }
		});
		document.getSelection()!.selectAllChildren(log.querySelector('li')!);
		Object.defineProperty(log, 'scrollHeight', { value: 1400 });
		log.scrollTop = 0;
		await rerender({ mode: 'transcribe', transcript: seed(11), onError: vi.fn(), reader: true });
		expect(log.scrollTop).toBe(0);
		document.getSelection()!.removeAllRanges();
	});

	// Review U12: each paragraph says the language it was captioned in, as the overlay does, so
	// Narrator reads it with that language's voice and Arabic lays out right to left. A line
	// restored from a recovery file was captioned in a language nobody recorded.
	it('gives each paragraph and the turn being spoken their caption language', async () => {
		const { options } = await import('./stores');
		const { DEFAULT_START_OPTIONS } = await import('./startOptions');
		const previous = get(options);
		options.set({
			...DEFAULT_START_OPTIONS,
			mode: 'translate',
			provider: 'gemini',
			targetLanguage: 'ar'
		});
		try {
			seed(1);
			pushCaption({ ...interim, turnId: 5, text: 'مرحبا بكم', final: true });
			pushCaption({ ...interim, turnId: 6, text: 'والميزانية', startMs: 9000, endMs: 9500 });
			const { getByText } = mount(get(transcript), { live: true, reader: true });
			for (const text of ['مرحبا بكم', 'والميزانية']) {
				const caption = getByText(text, { selector: '.text' });
				expect(caption).toHaveAttribute('lang', 'ar');
				expect(caption).toHaveAttribute('dir', 'rtl');
			}
			const restored = getByText('utterance 1', { selector: '.text' });
			expect(restored).toHaveAttribute('lang', '');
			expect(restored).toHaveAttribute('dir', 'auto');
		} finally {
			options.set(previous);
		}
	});

	it('selects only the log on Ctrl+A', async () => {
		const { getByRole } = mount(seed(2), { reader: true });
		const log = getByRole('region', { name: 'Transcript' });
		await fireEvent.keyDown(log, { key: 'a', ctrlKey: true });
		const selected = document.getSelection()!.toString();
		expect(selected).toContain('utterance 2');
		expect(selected).not.toContain('recovery copy');
		document.getSelection()!.removeAllRanges();
	});
});
