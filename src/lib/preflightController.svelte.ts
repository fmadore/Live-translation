import { get } from 'svelte/store';
import { api } from './tauri';
import { asStatus, describeError } from './errors';
import { t } from './i18n';
import { validateDevices } from './audioDevices';
import { micLevel, systemLevel, options, statusMessage } from './stores';
import type { AudioDevice, AudioLevel, AudioTestUpdate, OnDeviceReadiness } from './types';

/** Capture preflight and signal lifetime, independent of the operator's layout. */
export function createPreflightController(desktop: boolean, locked: () => boolean, port = api) {
	const api = port;
	let microphones = $state<AudioDevice[]>([]);
	let outputs = $state<AudioDevice[]>([]);
	let refreshing = $state(false);
	let loaded = false;
	let disposed = false;
	let refreshAgain = false;
	let localReadiness = $state<OnDeviceReadiness | null>(null);
	// ---- Pre-flight audio check -------------------------------------------------
	// A source counts as arriving while it has been above the noise floor recently. Driven by
	// the level events themselves, so nothing polls while the window sits idle.
	const SIGNAL_RMS = 0.02;
	const SIGNAL_HOLD_MS = 3000;
	let micSignal = $state(false);
	let systemSignal = $state(false);
	let micSignalTimer: ReturnType<typeof setTimeout> | undefined;
	let systemSignalTimer: ReturnType<typeof setTimeout> | undefined;

	function noteLevel(level: AudioLevel) {
		if (level.source === 'microphone') {
			micLevel.set(level);
			if (level.rms <= SIGNAL_RMS) return;
			micSignal = true;
			micVerified = true;
			clearTimeout(micSignalTimer);
			micSignalTimer = setTimeout(() => (micSignal = false), SIGNAL_HOLD_MS);
		} else {
			systemLevel.set(level);
			if (level.rms <= SIGNAL_RMS) return;
			systemSignal = true;
			systemVerified = true;
			clearTimeout(systemSignalTimer);
			systemSignalTimer = setTimeout(() => (systemSignal = false), SIGNAL_HOLD_MS);
		}
	}

	// ---- Preflight audio test ---------------------------------------------------
	// Levels only exist while something is capturing, so the idle sheet cannot observe the
	// room on its own. Rather than implying that it is listening, it offers a deliberate
	// level-only test: the same devices a session would open, every sample discarded, no
	// provider contacted and nothing billed or stored. See `SessionManager::start_test`.
	let audioTesting = $state(false);
	let audioTestBusy = $state(false);
	// Latched once a source has genuinely been heard, so the tick survives the test ending.
	// Dropped whenever the operator changes what is under test.
	let micVerified = $state(false);
	let systemVerified = $state(false);

	async function startAudioTest() {
		if (!desktop || audioTestBusy || audioTesting || locked()) return;
		audioTestBusy = true;
		statusMessage.set('');
		try {
			const selected = get(options);
			await api.startAudioTest(
				selected.source,
				selected.micDeviceId ?? selected.micDeviceName ?? null,
				selected.systemDeviceId ?? null
			);
		} catch (e) {
			statusMessage.set(asStatus(e));
		} finally {
			audioTestBusy = false;
		}
	}

	async function stopAudioTest() {
		if (!desktop) return;
		audioTestBusy = true;
		try {
			await api.stopAudioTest();
		} catch (e) {
			statusMessage.set(asStatus(e));
		} finally {
			audioTestBusy = false;
		}
	}

	/** A running test holds one specific device. Once the operator changes the source, the
	 *  device or the provider, that probe is measuring something they are no longer asking
	 *  about — so release it, and drop the verdict along with it. */
	function invalidateAudioTest() {
		micVerified = false;
		systemVerified = false;
		micSignal = false;
		systemSignal = false;
		clearTimeout(micSignalTimer);
		clearTimeout(systemSignalTimer);
		if (audioTesting) void stopAudioTest();
	}

	function validateSelection() {
		if (!loaded || locked() || audioTesting || audioTestBusy) return;
		const current = get(options);
		const next = validateDevices(current, microphones, outputs);
		if (
			current.micDeviceId !== next.micDeviceId ||
			current.systemDeviceId !== next.systemDeviceId ||
			current.micDeviceName !== next.micDeviceName
		) {
			if (
				!get(statusMessage) &&
				(((current.micDeviceId || current.micDeviceName) && !next.micDeviceId) ||
					(current.systemDeviceId && !next.systemDeviceId))
			) {
				statusMessage.set(get(t).devices.idleFallback);
			}
			invalidateAudioTest();
			options.set(next);
		}
	}

	async function refresh() {
		if (!desktop || disposed) return;
		if (refreshing) {
			refreshAgain = true;
			return;
		}
		refreshing = true;
		try {
			do {
				refreshAgain = false;
				const [mics, render] = await Promise.all([api.listMicrophones(), api.listOutputs()]);
				if (disposed) return;
				if (loaded && JSON.stringify([mics, render]) !== JSON.stringify([microphones, outputs])) {
					micVerified = false;
					systemVerified = false;
				}
				microphones = mics;
				outputs = render;
				loaded = true;
				validateSelection();
			} while (refreshAgain && !disposed);
		} catch (e) {
			if (!disposed) statusMessage.set(asStatus(e));
		} finally {
			refreshing = false;
		}
	}

	async function refreshLocalReadiness() {
		if (!desktop) return;
		try {
			localReadiness = await api.onDeviceReadiness();
		} catch (e) {
			localReadiness = {
				ready: false,
				engine: 'none',
				state: 'check-failed',
				canPrepare: false,
				detail: describeError(e, get(t))
			};
		}
	}

	function applyAudioTest(update: AudioTestUpdate) {
		audioTesting = update.active;
		if (!update.active) {
			micSignal = false;
			systemSignal = false;
			clearTimeout(micSignalTimer);
			clearTimeout(systemSignalTimer);
			if (update.message) statusMessage.set(update.message);
		}
	}

	function dispose() {
		disposed = true;
		clearTimeout(micSignalTimer);
		clearTimeout(systemSignalTimer);
		if (desktop && audioTesting) void stopAudioTest();
	}

	return {
		get outputs() {
			return outputs;
		},
		get refreshing() {
			return refreshing;
		},
		validateSelection,
		get microphones() {
			return microphones;
		},
		get localReadiness() {
			return localReadiness;
		},
		get micSignal() {
			return micSignal;
		},
		get systemSignal() {
			return systemSignal;
		},
		get audioTesting() {
			return audioTesting;
		},
		get audioTestBusy() {
			return audioTestBusy;
		},
		get micVerified() {
			return micVerified;
		},
		get systemVerified() {
			return systemVerified;
		},
		noteLevel,
		applyAudioTest,
		startAudioTest,
		stopAudioTest,
		invalidateAudioTest,
		refresh,
		refreshLocalReadiness,
		dispose
	};
}
