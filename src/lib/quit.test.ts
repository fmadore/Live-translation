import { beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';

const mocks = vi.hoisted(() => ({
	saveTranscript: vi.fn(),
	clearRecovery: vi.fn(),
	ackClose: vi.fn(),
	confirmClose: vi.fn()
}));

vi.mock('./tauri', () => ({
	isTauri: () => true,
	api: {
		saveTranscript: mocks.saveTranscript,
		clearRecovery: mocks.clearRecovery,
		ackClose: mocks.ackClose,
		confirmClose: mocks.confirmClose
	}
}));

const { prepareClose, resolveClose } = await import('./quit');
const { applyStatus, clearTranscript, pushCaption, savedPath, transcript, transcriptDirty } =
	await import('./stores');
import type { Caption } from './types';

function caption(turnId: number, text: string, final = true): Caption {
	return { turnId, text, sourceText: `src ${text}`, final, origin: 'microphone' };
}

/** Put the session into the state the operator sees while captions are streaming. */
function startRunning() {
	applyStatus({ state: 'running', origin: 'microphone' });
}

beforeEach(() => {
	vi.clearAllMocks();
	applyStatus({ state: 'idle' });
	clearTranscript();
	mocks.saveTranscript.mockResolvedValue('C:\\Docs\\Live-translation\\transcript.md');
	mocks.clearRecovery.mockResolvedValue(undefined);
	mocks.ackClose.mockResolvedValue(undefined);
	mocks.confirmClose.mockResolvedValue(undefined);
});

describe('preparing to close', () => {
	it('quits straight away when nothing is running and nothing is unsaved', async () => {
		const stop = vi.fn().mockResolvedValue(undefined);

		await expect(prepareClose(stop)).resolves.toEqual({ endedSession: false, prompt: false });
		expect(stop).not.toHaveBeenCalled();
		expect(mocks.confirmClose).toHaveBeenCalledTimes(1);
	});

	// The core releases the window if nothing claims the close within a few seconds, and a
	// session stop takes longer than that — so the claim has to go out before the drain.
	it('claims the close before it starts stopping anything', async () => {
		startRunning();
		const order: string[] = [];
		mocks.ackClose.mockImplementation(async () => void order.push('ack'));
		const stop = vi.fn().mockImplementation(async () => void order.push('stop'));

		await prepareClose(stop);

		expect(order).toEqual(['ack', 'stop']);
	});

	// Criterion: an active-session shutdown stops capture and drains the provider tails before
	// the document is called finished.
	it('stops the session first, then quits when the drain produced nothing to save', async () => {
		startRunning();
		const stop = vi.fn().mockImplementation(async () => applyStatus({ state: 'idle' }));

		await expect(prepareClose(stop)).resolves.toEqual({ endedSession: true, prompt: false });
		expect(stop).toHaveBeenCalledTimes(1);
		expect(mocks.confirmClose).toHaveBeenCalledTimes(1);
	});

	// The last turn of a session routinely arrives without an explicit turn-complete. It is
	// still the operator's text, and it has to be in the document before the question is asked.
	it('commits the in-flight turn the drain produced and asks about it', async () => {
		startRunning();
		pushCaption(caption(7, 'the closing remarks', false));
		const stop = vi.fn().mockResolvedValue(undefined);

		await expect(prepareClose(stop)).resolves.toEqual({ endedSession: true, prompt: true });
		expect(get(transcript).map((l) => l.text)).toEqual(['the closing remarks']);
		expect(mocks.confirmClose).not.toHaveBeenCalled();
	});

	it('asks before quitting on unsaved lines from an already-stopped session', async () => {
		pushCaption(caption(1, 'kept in memory only'));
		const stop = vi.fn().mockResolvedValue(undefined);

		await expect(prepareClose(stop)).resolves.toEqual({ endedSession: false, prompt: true });
		expect(mocks.confirmClose).not.toHaveBeenCalled();
	});

	// A provider that never finishes flushing must not leave the operator with a window that
	// will not close and no explanation.
	it('gives up on a stop that never returns and still asks the question', async () => {
		vi.useFakeTimers();
		try {
			startRunning();
			pushCaption(caption(1, 'unsaved'));
			const stop = vi.fn().mockImplementation(() => new Promise<void>(() => {}));

			const pending = prepareClose(stop, 8000);
			await vi.advanceTimersByTimeAsync(8000);

			await expect(pending).resolves.toEqual({ endedSession: true, prompt: true });
		} finally {
			vi.useRealTimers();
		}
	});

	it('survives a stop that rejects rather than turning it into an unhandled rejection', async () => {
		startRunning();
		const stop = vi.fn().mockRejectedValue(new Error('capture thread panicked'));

		await expect(prepareClose(stop, 50)).resolves.toEqual({ endedSession: true, prompt: false });
		expect(mocks.confirmClose).toHaveBeenCalledTimes(1);
	});
});

describe('answering the prompt', () => {
	it('writes Markdown, records the save, retires the spool, and quits', async () => {
		pushCaption(caption(1, 'worth keeping'));

		await resolveClose('save');

		const [content, filename] = mocks.saveTranscript.mock.calls[0];
		expect(content).toContain('worth keeping');
		expect(filename).toMatch(/\.md$/);
		expect(get(transcriptDirty)).toBe(false);
		expect(get(savedPath)).toBe('C:\\Docs\\Live-translation\\transcript.md');
		expect(mocks.clearRecovery).toHaveBeenCalledTimes(1);
		expect(mocks.confirmClose).toHaveBeenCalledTimes(1);
	});

	// Quitting on a write that did not land would lose exactly what the operator asked to keep.
	it('does not quit when the write fails', async () => {
		pushCaption(caption(1, 'worth keeping'));
		mocks.saveTranscript.mockRejectedValue(new Error('could not create "D:\\\\Docs"'));

		await expect(resolveClose('save')).rejects.toThrow('could not create');
		expect(mocks.confirmClose).not.toHaveBeenCalled();
		expect(get(transcriptDirty)).toBe(true);
	});

	it('discards by deleting the spool and quitting without writing a transcript', async () => {
		pushCaption(caption(1, 'not worth keeping'));

		await resolveClose('discard');

		expect(mocks.saveTranscript).not.toHaveBeenCalled();
		expect(mocks.clearRecovery).toHaveBeenCalledTimes(1);
		expect(mocks.confirmClose).toHaveBeenCalledTimes(1);
	});

	// The spool is a convenience; failing to remove it is not a reason to refuse to quit after
	// the operator has already said they do not want the text.
	it('still quits when the spool cannot be deleted', async () => {
		pushCaption(caption(1, 'not worth keeping'));
		mocks.clearRecovery.mockRejectedValue(new Error('file is locked'));

		await expect(resolveClose('discard')).resolves.toBeUndefined();
		expect(mocks.confirmClose).toHaveBeenCalledTimes(1);
	});
});
