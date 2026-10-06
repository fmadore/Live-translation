import { get } from 'svelte/store';
import { api } from './tauri';
import { asStatus, describeError } from './errors';
import { t } from './i18n';
import { validateDevices } from './audioDevices';
import { micLevel, systemLevel, options, statusMessage } from './stores';
import { providerRequiresKey } from './types';
import type {
	AudioDevice,
	AudioLevel,
	AudioTestUpdate,
	OnDeviceReadiness,
	Provider,
	StartOptions,
	WhisperCpuSupport,
	WhisperModelInfo
} from './types';

export interface PreflightGates {
	/** Setup is locked: a session, a start or a profile load is under way. Nothing is tested or
	 *  re-validated then. The Test audio button is disabled on exactly this, so a click the
	 *  button accepts is never silently refused here. */
	locked: () => boolean;
	/** Idle validation must leave the saved devices alone, on top of `locked`: while a capture
	 *  failure is on screen, its Retry still needs the endpoint the operator chose. A test is
	 *  allowed then — checking another microphone is exactly what the banner invites. */
	holdSelection?: () => boolean;
}

/** Capture preflight and signal lifetime, independent of the operator's layout. */
export function createPreflightController(
	desktop: boolean,
	{ locked, holdSelection = () => false }: PreflightGates,
	port = api
) {
	const api = port;
	let microphones = $state<AudioDevice[]>([]);
	let outputs = $state<AudioDevice[]>([]);
	let applications = $state<import('./types').CaptureApplication[]>([]);
	let applicationCaptureSupported = $state<boolean | null>(null);
	let refreshingApplications = $state(false);
	function applicationReady(selected: import('./types').StartOptions): boolean {
		if (
			selected.source === 'microphone' ||
			selected.provider === 'ondevice' ||
			selected.systemCapture?.kind !== 'application'
		)
			return true;
		const process = selected.systemCapture.process;
		return (
			applicationCaptureSupported === true &&
			!!process &&
			applications.some(
				(app) => app.process.pid === process.pid && app.process.createdAt === process.createdAt
			)
		);
	}
	async function refreshApplications() {
		if (!desktop || disposed || refreshingApplications) return;
		refreshingApplications = true;
		try {
			const result = await api.listApplications();
			if (disposed) return;
			applications = result.applications;
			applicationCaptureSupported = result.supported;
		} catch (error) {
			if (!disposed) statusMessage.set(asStatus(error));
		} finally {
			refreshingApplications = false;
		}
	}
	let refreshing = $state(false);
	let loaded = false;
	let disposed = false;
	let refreshAgain = false;
	let localReadiness = $state<OnDeviceReadiness | null>(null);
	// Null until the core has answered, which blocks a Whisper start in the meantime. A browser
	// preview has no engine to protect, so it counts as supported.
	let whisperCpu = $state<WhisperCpuSupport | null>(
		desktop ? null : { supported: true, missing: [] }
	);
	// The engine whose stored key the key panel last confirmed. Tied to that engine rather than
	// kept as a bare flag, so switching engines can never borrow the previous one's answer while
	// the panel is still asking the keychain about the new one.
	let keyFor = $state<Provider | null>(null);

	/** The key panel's answer for the engine it is showing. */
	function noteKey(provider: Provider, available: boolean) {
		keyFor = available ? provider : null;
	}

	/** Whether `selected`'s engine could start now: its key is stored, its Whisper model is
	 *  installed on a processor that can run it, or the built-in demonstration is ready. Read
	 *  through a `$derived` on the page, so it follows every input as it changes. */
	function engineReady(selected: StartOptions, models: readonly WhisperModelInfo[]): boolean {
		if (providerRequiresKey(selected.provider)) return keyFor === selected.provider;
		if (selected.provider === 'whisper') {
			// Until the core has said this processor can run it, Whisper cannot start.
			const model = selected.whisperModel ?? 'base';
			return (
				whisperCpu?.supported === true &&
				models.some((m) => m.id === model && m.installed && !m.downloading)
			);
		}
		return localReadiness?.ready ?? false;
	}
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
		if (!applicationReady(get(options))) {
			statusMessage.set(get(t).applications.missing);
			return;
		}
		audioTestBusy = true;
		statusMessage.set('');
		try {
			const selected = get(options);
			await api.startAudioTest(
				selected.source,
				selected.micDeviceId ?? selected.micDeviceName ?? null,
				selected.systemDeviceId ?? null,
				selected.systemCapture
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
		if (!loaded || locked() || holdSelection() || audioTesting || audioTestBusy) return;
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

	/** Ask once whether this processor can run local Whisper. The answer cannot change while the
	 *  app runs. A failed question is treated as supported: the core checks again at Start and
	 *  refuses with its own sentence, so guessing "no" would only hide a working engine. */
	async function refreshWhisperCpu() {
		if (!desktop) return;
		try {
			whisperCpu = await api.whisperCpuSupport();
		} catch {
			whisperCpu = { supported: true, missing: [] };
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
		applicationReady,
		engineReady,
		noteKey,
		get applications() {
			return applications;
		},
		get applicationCaptureSupported() {
			return applicationCaptureSupported;
		},
		get refreshingApplications() {
			return refreshingApplications;
		},
		refreshApplications,
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
		get whisperCpu() {
			return whisperCpu;
		},
		/** The processor is known to lack what local Whisper needs. */
		get whisperRefused() {
			return whisperCpu?.supported === false;
		},
		/** The first few absent instruction sets, for the one-line reason. A pre-AVX processor
		 *  lacks most of the list, and all of it would wrap across a card. */
		get whisperCpuMissing() {
			const missing = whisperCpu?.missing ?? [];
			return missing.slice(0, 3).join(', ') + (missing.length > 3 ? '…' : '');
		},
		refreshWhisperCpu,
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
export type PreflightController = ReturnType<typeof createPreflightController>;
