import { get } from 'svelte/store';
import { beforeEach, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
	acknowledgeClose: vi.fn(),
	endsLiveSession: vi.fn(),
	prepareClose: vi.fn(),
	resolveClose: vi.fn(),
	showOperator: vi.fn(),
	hideToTray: vi.fn()
}));
vi.mock('./quit', () => mocks);
vi.mock('./tauri', () => ({ isTauri: () => false, api: mocks }));
import { createQuitController } from './quitController.svelte';
import { closeToTray, trayHideExplained } from './stores';

beforeEach(() => {
	vi.resetAllMocks();
	closeToTray.set(false);
	mocks.prepareClose.mockResolvedValue({ prompt: true, endedSession: true });
});

it('acknowledges repeated close requests without repeating the shutdown', async () => {
	const controller = createQuitController({ stop: vi.fn(), toggleOverlayVisible: vi.fn() });
	await controller.onCloseRequested();
	await controller.onCloseRequested();
	expect(mocks.acknowledgeClose).toHaveBeenCalledTimes(2);
	expect(mocks.prepareClose).toHaveBeenCalledTimes(1);
	expect(controller.closePrompt).toBe(true);
});

it('asks before stopping a live session, then retains unsaved text on save failure', async () => {
	mocks.endsLiveSession.mockReturnValue(true);
	const controller = createQuitController({ stop: vi.fn(), toggleOverlayVisible: vi.fn() });
	await controller.onTrayCommand('quit');
	expect(controller.sessionPrompt).toBe(true);
	expect(controller.sessionPromptFromTray).toBe(true);
	expect(mocks.prepareClose).not.toHaveBeenCalled();
	await controller.onSessionChoice(true);
	expect(controller.sessionPrompt).toBe(false);
	expect(controller.closePrompt).toBe(true);
	mocks.resolveClose.mockRejectedValue(new Error('Disk full'));
	await controller.onCloseChoice('save');
	expect(controller.closePrompt).toBe(true);
	expect(controller.closeError).toBe('Disk full');
	expect(controller.closeAnswering).toBeNull();
});

// U11: kept as it was caught, so the prompt words it in whatever language is active.
it('keeps a failed save as the core reported it, for the prompt to word', async () => {
	const controller = createQuitController({ stop: vi.fn(), toggleOverlayVisible: vi.fn() });
	await controller.onCloseRequested();
	const failure = { id: 'error.transcriptWrite', detail: 'disk full' };
	mocks.resolveClose.mockRejectedValue(failure);
	await controller.onCloseChoice('save');
	expect(controller.closeError).toEqual(failure);

	await controller.onCloseChoice('cancel');
	expect(controller.closePrompt).toBe(false);
	expect(controller.closeError).toBe('');
});

// U9: one answer at a time. A Cancel while Save is writing used to close the prompt over a quit
// that was still going ahead, and a Discard started a second one.
it('takes no other answer while one is being carried out', async () => {
	const controller = createQuitController({ stop: vi.fn(), toggleOverlayVisible: vi.fn() });
	await controller.onCloseRequested();
	let finish!: (quit: boolean) => void;
	mocks.resolveClose.mockReturnValue(new Promise<boolean>((resolve) => (finish = resolve)));

	const saving = controller.onCloseChoice('save');
	expect(controller.closeAnswering).toBe('save');
	await controller.onCloseChoice('cancel');
	await controller.onCloseChoice('discard');
	expect(controller.closePrompt).toBe(true);
	expect(mocks.resolveClose).toHaveBeenCalledTimes(1);

	finish(true);
	await saving;
	expect(controller.closeAnswering).toBeNull();
	expect(controller.closePrompt).toBe(false);
});

it('the tray Quit with nothing live claims the close and goes straight to the unsaved question', async () => {
	mocks.endsLiveSession.mockReturnValue(false);
	const stop = vi.fn();
	const controller = createQuitController({ stop, toggleOverlayVisible: vi.fn() });
	await controller.onTrayCommand('quit');
	expect(mocks.acknowledgeClose).toHaveBeenCalledOnce();
	expect(controller.sessionPrompt).toBe(false);
	expect(mocks.prepareClose).toHaveBeenCalledExactlyOnceWith(stop);
	expect(mocks.showOperator).toHaveBeenCalledOnce();
	expect(controller.closePrompt).toBe(true);
});

it('choosing Quit on the tray explanation quits without marking the explanation as given', async () => {
	closeToTray.set(true);
	trayHideExplained.set(false);
	const controller = createQuitController({ stop: vi.fn(), toggleOverlayVisible: vi.fn() });
	await controller.onCloseRequested();
	expect(controller.hidePrompt).toBe(true);
	await controller.onHideChoice('quit');
	expect(controller.hidePrompt).toBe(false);
	expect(mocks.hideToTray).not.toHaveBeenCalled();
	expect(mocks.prepareClose).toHaveBeenCalledOnce();
	expect(get(trayHideExplained)).toBe(false);
});

it('Escape on the tray explanation neither hides nor quits, and still owes the explanation', async () => {
	closeToTray.set(true);
	trayHideExplained.set(false);
	const controller = createQuitController({ stop: vi.fn(), toggleOverlayVisible: vi.fn() });
	await controller.onCloseRequested();
	await controller.onHideChoice('cancel');
	expect(controller.hidePrompt).toBe(false);
	expect(mocks.hideToTray).not.toHaveBeenCalled();
	expect(mocks.prepareClose).not.toHaveBeenCalled();
	expect(get(trayHideExplained)).toBe(false);
	await controller.onHideChoice('hide');
	expect(mocks.hideToTray).toHaveBeenCalledOnce();
	expect(get(trayHideExplained)).toBe(true);
});

it('a second close request while the session question is up claims the close but asks nothing new', async () => {
	mocks.endsLiveSession.mockReturnValue(true);
	const controller = createQuitController({ stop: vi.fn(), toggleOverlayVisible: vi.fn() });
	await controller.onCloseRequested();
	expect(controller.sessionPrompt).toBe(true);
	expect(controller.sessionPromptFromTray).toBe(false);
	await controller.onTrayCommand('quit');
	expect(mocks.acknowledgeClose).toHaveBeenCalledTimes(2);
	expect(controller.sessionPromptFromTray).toBe(false);
	expect(mocks.showOperator).toHaveBeenCalledOnce();
	expect(mocks.prepareClose).not.toHaveBeenCalled();
	// Keeping the session withdraws the question and quits nothing.
	await controller.onSessionChoice(false);
	expect(controller.sessionPrompt).toBe(false);
	expect(mocks.prepareClose).not.toHaveBeenCalled();
});
