import { get } from 'svelte/store';
import { api } from './tauri';
import { asStatus, describeError } from './errors';
import { t } from './i18n';
import { micLevel, systemLevel, options, statusMessage } from './stores';
import type { AudioDevice, AudioLevel, AudioTestUpdate, OnDeviceReadiness } from './types';

/** Capture preflight and signal lifetime, independent of the operator's layout. */
export function createPreflightController(desktop: boolean, locked: () => boolean, port = api) {
	const api = port;
	let microphones = $state<AudioDevice[]>([]);
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
			await api.startAudioTest(get(options).source, get(options).micDeviceName ?? null);
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

	async function refresh() {
		if (!desktop) return;
		if (get(options).provider === 'ondevice') {
			microphones = [];
			return;
		}
		try {
			microphones = await api.listMicrophones();
			// Options persist across launches, so a remembered device may be gone (unplugged,
			// renamed). Falling back to the system default beats failing at session start.
			const name = get(options).micDeviceName;
			if (name && !microphones.some((d) => d.name === name)) {
				options.update((current) => ({ ...current, micDeviceName: null }));
			}
		} catch (e) {
			statusMessage.set(asStatus(e));
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
		clearTimeout(micSignalTimer);
		clearTimeout(systemSignalTimer);
		if (desktop && audioTesting) void stopAudioTest();
	}

	return {
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
