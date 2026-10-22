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
import { closeToTray } from './stores';

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
