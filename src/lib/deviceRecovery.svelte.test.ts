import { expect, it, vi } from 'vitest';
import { fireEvent, render, waitFor } from '@testing-library/svelte';
import { get } from 'svelte/store';

const native = vi.hoisted(() => ({
	handlers: {} as Record<string, (value?: unknown) => void>,
	startSession: vi.fn(),
	stopSession: vi.fn()
}));
vi.mock('./tauri', () => ({
	isTauri: () => true,
	api: new Proxy(
		{
			startSession: native.startSession,
			stopSession: native.stopSession,
			listMicrophones: vi.fn().mockResolvedValue([{ id: 'mic-1', name: 'Mic', isDefault: true }]),
			listOutputs: vi.fn().mockResolvedValue([{ id: 'render-1', name: 'Dock', isDefault: true }]),
			listApplications: vi.fn().mockResolvedValue({ supported: true, applications: [] }),
			onDeviceReadiness: vi.fn().mockResolvedValue({ ready: true }),
			readRecovery: vi.fn().mockResolvedValue(null)
		},
		{
			get: (target, key) =>
				key in target ? target[key as keyof typeof target] : vi.fn().mockResolvedValue(undefined)
		}
	),
	on: new Proxy(
		{},
		{
			get: (_, key) => (handler: (value?: unknown) => void) => {
				native.handlers[String(key)] = handler;
				return Promise.resolve(() => {
					delete native.handlers[String(key)];
				});
			}
		}
	)
}));
vi.mock('./textScale', () => ({ followTextScale: async () => () => {} }));
vi.mock('./overlayController.svelte', () => ({
	createOverlayController: () => ({
		captionFaces: [],
		initialize() {},
		pushOverlayConfig() {},
		applyState() {},
		overlayVisible: true,
		moveOverlay: false
	})
}));
import Page from '../routes/+page.svelte';
import { options, applyStatus, transcript, pushCaption, clearTranscript } from './stores';
import { locale } from './i18n';

it('reselecting an application stops capture without widening scope or discarding text', async () => {
	vi.clearAllMocks();
	locale.set('en');
	clearTranscript();
	applyStatus({ state: 'idle' });
	options.set({
		source: 'both',
		provider: 'gemini',
		mode: 'translate',
		targetLanguage: 'en',
		systemCapture: { kind: 'application', process: { pid: 123, createdAt: 'old' } }
	});
	native.stopSession.mockImplementation(async () => {
		applyStatus({ state: 'idle' });
	});
	const view = render(Page);
	await waitFor(() => expect(native.handlers.status).toBeTypeOf('function'));
	native.handlers.status({ state: 'running', origin: 'microphone' });
	pushCaption({
		origin: 'microphone',
		turnId: 1,
		text: 'Keep application transcript',
		sourceText: '',
		final: true,
		startMs: 0,
		endMs: 100
	});
	native.handlers.status({
		state: 'error',
		origin: 'system',
		message: { id: 'error.systemCapture', detail: 'Application closed' }
	});
	await waitFor(() =>
		expect(view.getByRole('button', { name: 'Stop and select an application' })).toBeInTheDocument()
	);
	expect(view.queryByRole('button', { name: 'Stop and retry with default device' })).toBeNull();
	await fireEvent.click(view.getByRole('button', { name: 'Stop and select an application' }));
	await waitFor(() =>
		expect(get(options).systemCapture).toEqual({ kind: 'application', process: null })
	);
	expect(native.stopSession).toHaveBeenCalledOnce();
	expect(native.startSession).not.toHaveBeenCalled();
	expect(get(transcript)[0].text).toBe('Keep application transcript');
	view.unmount();
});

it('keeps the failed endpoint until an explicit fallback and drains before restarting', async () => {
	vi.clearAllMocks();
	locale.set('en');
	clearTranscript();
	options.set({
		source: 'both',
		provider: 'gemini',
		mode: 'translate',
		targetLanguage: 'en',
		micDeviceId: 'mic-1',
		systemDeviceId: 'render-1'
	});
	applyStatus({ state: 'idle' });
	const order: string[] = [];
	native.stopSession.mockImplementation(async () => {
		order.push('stop');
		applyStatus({ state: 'idle' });
	});
	native.startSession.mockImplementation(async () => {
		order.push('start');
	});
	const view = render(Page);
	await waitFor(() => expect(native.handlers.status).toBeTypeOf('function'));
	native.handlers.status({ state: 'running', origin: 'microphone' });
	pushCaption({
		origin: 'microphone',
		turnId: 1,
		text: 'Keep this transcript',
		sourceText: '',
		final: true,
		startMs: 0,
		endMs: 100
	});
	native.handlers.status({
		state: 'error',
		origin: 'system',
		message: { id: 'error.systemCapture', detail: 'Dock unplugged' }
	});
	await waitFor(() =>
		expect(view.getByRole('button', { name: 'Stop and retry with default device' })).toBeDefined()
	);
	expect(get(options).systemDeviceId).toBe('render-1');
	await fireEvent.click(view.getByRole('button', { name: 'Stop and retry with default device' }));
	await waitFor(() => expect(native.startSession).toHaveBeenCalled());
	expect(order).toEqual(['stop', 'start']);
	expect(native.startSession).toHaveBeenCalledWith(
		expect.objectContaining({ micDeviceId: 'mic-1', systemDeviceId: null })
	);
	expect(get(transcript).some((line) => line.text === 'Keep this transcript')).toBe(true);
	view.unmount();
});
