import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import { api } from './tauri';
import { createPreflightController } from './preflightController.svelte';
import { createOverlayController } from './overlayController.svelte';
import { options, overlayFontSize, overlayCaptionWidth, statusMessage } from './stores';

beforeEach(() => {
	statusMessage.set('');
});
afterEach(() => {
	vi.useRealTimers();
});

it('expires live signal while retaining verification until the input changes', () => {
	vi.useFakeTimers();
	const probe = createPreflightController(false, () => false);
	probe.noteLevel({ source: 'microphone', rms: 0.2, peak: 0.4 });
	expect(probe.micSignal).toBe(true);
	vi.advanceTimersByTime(3000);
	expect(probe.micSignal).toBe(false);
	expect(probe.micVerified).toBe(true);
	probe.invalidateAudioTest();
	expect(probe.micVerified).toBe(false);
	probe.dispose();
	expect(vi.getTimerCount()).toBe(0);
});

it('prevents overlapping test starts and leaves native events authoritative', async () => {
	let release!: () => void;
	const startAudioTest = vi.fn(
		() =>
			new Promise<void>((resolve) => {
				release = resolve;
			})
	);
	const probe = createPreflightController(true, () => false, { ...api, startAudioTest });
	const starting = probe.startAudioTest();
	await probe.startAudioTest();
	expect(startAudioTest).toHaveBeenCalledTimes(1);
	expect(probe.audioTestBusy).toBe(true);
	probe.applyAudioTest({ active: true });
	probe.applyAudioTest({ active: false, message: 'Device disconnected' });
	release();
	await starting;
	expect(probe.audioTesting).toBe(false);
	expect(probe.audioTestBusy).toBe(false);
	expect(get(statusMessage)).toBe('Device disconnected');
});

it('falls back to the default microphone when a saved device disappears', async () => {
	options.update((value) => ({ ...value, provider: 'gemini', micDeviceName: 'Unplugged' }));
	const probe = createPreflightController(true, () => false, {
		...api,
		listMicrophones: vi.fn().mockResolvedValue([{ name: 'Available', isDefault: true }])
	});
	await probe.refresh();
	expect(get(options).micDeviceName).toBeNull();
	expect(probe.microphones).toHaveLength(1);
});

it('keeps confirmed move mode when the native click-through command fails', async () => {
	const controller = createOverlayController({
		...api,
		showOverlay: vi.fn().mockResolvedValue(undefined),
		setOverlayClickThrough: vi.fn().mockRejectedValue(new Error('Window unavailable'))
	});
	await controller.toggleMoveOverlay();
	expect(controller.moveOverlay).toBe(false);
	expect(get(statusMessage)).not.toBe('');
});

it('sends the full appearance after edits and handles failed config writes', async () => {
	const setOverlayConfig = vi.fn().mockResolvedValue(undefined);
	const controller = createOverlayController({ ...api, setOverlayConfig });
	controller.setFont(42);
	controller.setCaptionWidth(70);
	expect(setOverlayConfig).toHaveBeenLastCalledWith(
		expect.objectContaining({
			fontSize: get(overlayFontSize),
			captionWidth: get(overlayCaptionWidth),
			captionFace: expect.any(String),
			captionColour: expect.any(String),
			scrimColour: expect.any(String),
			scrimOpacity: expect.any(Number),
			interactive: false
		})
	);
	setOverlayConfig.mockRejectedValue(new Error('Window unavailable'));
	controller.setFont(44);
	await Promise.resolve();
	expect(get(statusMessage)).not.toBe('');
});
