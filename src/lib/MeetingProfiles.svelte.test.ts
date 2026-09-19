import { beforeEach, expect, it, vi } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import { get } from 'svelte/store';
import MeetingProfiles from './MeetingProfiles.svelte';
import { api } from './tauri';
import { createOverlayController } from './overlayController.svelte';
import { options, overlayFontSize, overlayPace } from './stores';
import { PROFILES_KEY } from './profiles';
vi.mock('./tauri', () => ({
	isTauri: () => true,
	api: {
		getOverlayPlacement: vi.fn(),
		setOverlayPlacement: vi.fn(),
		listMicrophones: vi.fn(),
		listOutputs: vi.fn(),
		setOverlayConfig: vi.fn()
	}
}));
beforeEach(() => {
	localStorage.removeItem(PROFILES_KEY);
	vi.mocked(api.getOverlayPlacement).mockResolvedValue({ x: 0, y: 0, width: 900, height: 240 });
	vi.mocked(api.setOverlayPlacement).mockResolvedValue();
	vi.mocked(api.listMicrophones).mockResolvedValue([]);
	vi.mocked(api.listOutputs).mockResolvedValue([]);
	vi.mocked(api.setOverlayConfig).mockResolvedValue();
});
it('saves and reloads setup, revalidates missing devices and excludes old process ids', async () => {
	const overlay = createOverlayController();
	vi.spyOn(overlay, 'initialize').mockImplementation(() => {});
	options.set({
		source: 'both',
		mode: 'translate',
		targetLanguage: 'fr',
		provider: 'gemini',
		micDeviceId: 'missing',
		systemCapture: { kind: 'application', process: { pid: 3, createdAt: 'old' } }
	});
	overlayFontSize.set(52);
	overlayPace.set('steady');
	const loaded = vi.fn().mockResolvedValue(undefined);
	const busy = vi.fn();
	const view = render(MeetingProfiles, { locked: false, overlay, onLoaded: loaded, onBusy: busy });
	await fireEvent.input(view.getByLabelText('Profile name'), { target: { value: 'Lecture hall' } });
	await fireEvent.click(view.getByText('Save current setup'));
	await waitFor(() => expect(view.getByText('Profile saved.')).toBeTruthy());
	expect(
		JSON.parse(localStorage.getItem(PROFILES_KEY)!)[0].options.systemCapture.process
	).toBeNull();
	overlayFontSize.set(30);
	overlayPace.set('immediate');
	await fireEvent.click(view.getByText('Load profile'));
	await waitFor(() => expect(loaded).toHaveBeenCalledOnce());
	expect(get(overlayFontSize)).toBe(52);
	expect(get(overlayPace)).toBe('steady');
	expect(get(options).micDeviceId).toBeNull();
	expect(api.setOverlayPlacement).toHaveBeenCalled();
	expect(view.getByText(/An audio device is unavailable/)).toBeTruthy();
	await fireEvent.click(view.getByText('Delete profile'));
	expect(JSON.parse(localStorage.getItem(PROFILES_KEY)!)).toHaveLength(1);
	await fireEvent.click(view.getByText('Delete permanently'));
	await waitFor(() => expect(JSON.parse(localStorage.getItem(PROFILES_KEY)!)).toHaveLength(0));
	view.unmount();
});
