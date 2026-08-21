<script lang="ts">
	import { onMount } from 'svelte';
	import { api, on, isTauri } from '$lib/tauri';
	import {
		sessionState,
		isRunning,
		statusMessage,
		applyStatus,
		hasKey,
		options,
		currentCaptions,
		transcript,
		micLevel,
		systemLevel,
		overlayFontSize,
		overlayPlaced,
		sessionStartedAt,
		pushCaption,
		beginSession
	} from '$lib/stores';
	import type {
		AudioDevice,
		AudioLevel,
		AudioSource,
		Caption,
		Origin,
		OutputMode,
		Provider,
		SessionState,
		TargetLanguage
	} from '$lib/types';
	import { clampOverlayFont, providerCanTranslate, providerRequiresKey } from '$lib/types';
	import { PROVIDER_META, estimateSessionCost, formatUsd } from '$lib/providers';
	import LevelMeter from '$lib/LevelMeter.svelte';
	import ApiKeyPanel from '$lib/ApiKeyPanel.svelte';
	import TranscriptMonitor from '$lib/TranscriptMonitor.svelte';

	let microphones = $state<AudioDevice[]>([]);
	let browserMode = $state(false);
	let sessionBusy = $state(false);
	const controlsLocked = $derived($isRunning || sessionBusy);

	// The on-device engine has no key panel to report readiness, so mark it ready here.
	// A cloud provider starts NOT ready: clearing the flag on the switch itself closes the
	// tick where the previous provider's `true` would leave Start enabled before the
	// remounted ApiKeyPanel has re-checked the keychain.
	const needsKey = $derived(providerRequiresKey($options.provider));
	$effect(() => {
		hasKey.set(!needsKey);
	});

	const meta = $derived(PROVIDER_META[$options.provider]);
	// Each mode is served by exactly two backends; step 04 lists the pair for the current one.
	const modeProviders = $derived<Provider[]>(
		$options.mode === 'translate' ? ['gemini', 'openai'] : ['mistral', 'ondevice']
	);

	// ---- Session clock ---------------------------------------------------------
	// Ticks only while a session is open; the effect's teardown stops it on stop/unmount.
	let clock = $state(Date.now());
	$effect(() => {
		if (!$isRunning) return;
		clock = Date.now();
		const id = setInterval(() => (clock = Date.now()), 1000);
		return () => clearInterval(id);
	});
	const elapsedMs = $derived($sessionStartedAt === null ? 0 : Math.max(0, clock - $sessionStartedAt));

	function formatElapsed(ms: number): string {
		const total = Math.floor(ms / 1000);
		const pad = (n: number) => String(n).padStart(2, '0');
		const seconds = total % 60;
		const minutes = Math.floor(total / 60) % 60;
		const hours = Math.floor(total / 3600);
		return hours > 0 ? `${hours}:${pad(minutes)}:${pad(seconds)}` : `${pad(minutes)}:${pad(seconds)}`;
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
			clearTimeout(micSignalTimer);
			micSignalTimer = setTimeout(() => (micSignal = false), SIGNAL_HOLD_MS);
		} else {
			systemLevel.set(level);
			if (level.rms <= SIGNAL_RMS) return;
			systemSignal = true;
			clearTimeout(systemSignalTimer);
			systemSignalTimer = setTimeout(() => (systemSignal = false), SIGNAL_HOLD_MS);
		}
	}

	const usesMic = $derived($options.source !== 'system');
	const usesSystem = $derived($options.source !== 'microphone');
	const audioReady = $derived((!usesMic || micSignal) && (!usesSystem || systemSignal));

	const audioTitle = $derived(
		$options.source === 'system' ? 'System audio' : $options.source === 'microphone' ? 'Room mic' : 'Audio'
	);
	const audioReadyDesc = $derived(
		$options.source === 'system'
			? 'WASAPI loopback is receiving sound'
			: $options.source === 'microphone'
				? 'The room mic is picking up sound'
				: 'Both the room mic and WASAPI loopback are receiving sound'
	);

	onMount(() => {
		browserMode = !isTauri();
		if (browserMode) return;

		void refresh();
		// Sync the overlay to the operator's current caption size on load.
		void api.setOverlayConfig({ fontSize: $overlayFontSize });

		const unlisteners: Array<Promise<() => void>> = [
			on.caption((c) => pushCaption(c)),
			on.level((l) => noteLevel(l)),
			on.status((s) => applyStatus(s)),
			// The overlay can be locked, placed and resized from its own toolbar; mirror that
			// back so the rail and the pre-flight check don't drift out of sync.
			on.overlayState((msg) => {
				if (msg.interactive === false) moveOverlay = false;
				if (msg.placed === true) overlayPlaced.set(true);
				if (typeof msg.fontSize === 'number' && Number.isFinite(msg.fontSize)) {
					overlayFontSize.set(clampOverlayFont(msg.fontSize));
				}
			})
		];

		return () => {
			clearTimeout(micSignalTimer);
			clearTimeout(systemSignalTimer);
			void Promise.all(unlisteners).then((fns) => fns.forEach((f) => f()));
		};
	});

	async function refresh() {
		try {
			microphones = await api.listMicrophones();
			// Options persist across launches, so a remembered device may be gone (unplugged,
			// renamed). Falling back to the system default beats failing at session start.
			const name = $options.micDeviceName;
			if (name && !microphones.some((d) => d.name === name)) {
				$options = { ...$options, micDeviceName: null };
			}
		} catch (e) {
			statusMessage.set(String(e));
		}
	}

	async function start() {
		if (sessionBusy || $isRunning) return;
		sessionBusy = true;
		statusMessage.set('');
		beginSession();
		try {
			await api.startSession($options);
		} catch (e) {
			statusMessage.set(String(e));
		} finally {
			sessionBusy = false;
		}
	}

	async function stop() {
		if (sessionBusy) return;
		sessionBusy = true;
		try {
			await api.stopSession();
		} catch (e) {
			statusMessage.set(String(e));
		} finally {
			sessionBusy = false;
		}
	}

	function setSource(s: AudioSource) {
		if (controlsLocked) return;
		$options = { ...$options, source: s };
	}

	function setTarget(t: TargetLanguage) {
		if (controlsLocked) return;
		// Translation picks the language the room reads; the on-device recognizer is told which
		// language to expect. Voxtral auto-detects, so it has nothing to set.
		if ($options.mode !== 'translate' && $options.provider !== 'ondevice') return;
		$options = { ...$options, targetLanguage: t };
	}

	function setProvider(p: Provider) {
		if (controlsLocked || p === $options.provider) return;
		// Each mode accepts only the backends that can serve it.
		if (providerCanTranslate(p) !== ($options.mode === 'translate')) return;
		$options = { ...$options, provider: p };
	}

	function setMode(mode: OutputMode) {
		if (controlsLocked || mode === $options.mode) return;
		$options = {
			...$options,
			mode,
			provider: mode === 'transcribe' ? 'mistral' : 'gemini'
		};
	}

	// Quick flip of the caption language — handy when speakers alternate.
	function flipDirection() {
		if ($options.mode !== 'translate' || controlsLocked) return;
		setTarget($options.targetLanguage === 'en' ? 'fr' : 'en');
	}

	// Caption size: update the store (persists) and push it live to the overlay.
	function setFont(size: number) {
		const clamped = clampOverlayFont(size);
		overlayFontSize.set(clamped);
		void api.setOverlayConfig({ fontSize: clamped, interactive: moveOverlay });
	}

	// Move mode: the overlay is click-through while captioning; this flips it into an
	// interactive drag region so it can be dragged/resized into place, then flipped back.
	// The overlay can also leave move mode on its own (its Enter/Escape keys), which arrives
	// as an overlayState event — so this flag is the single source of truth, never cached.
	let moveOverlay = $state(false);

	// The overlay window is created visible (tauri.conf.json), so the toggle starts on "Hide".
	// Blanking it covers a coffee break or a video clip without ending the session.
	let overlayVisible = $state(true);

	async function toggleMoveOverlay() {
		moveOverlay = !moveOverlay;
		try {
			await api.showOverlay(true);
			overlayVisible = true;
			await api.setOverlayClickThrough(!moveOverlay);
			await api.setOverlayConfig({ fontSize: $overlayFontSize, interactive: moveOverlay });
		} catch (e) {
			statusMessage.set(String(e));
		}
	}

	async function toggleOverlayVisible() {
		const next = !overlayVisible;
		try {
			await api.showOverlay(next);
			overlayVisible = next;
		} catch (e) {
			statusMessage.set(String(e));
		}
	}

	// ---- Display labels ---------------------------------------------------------

	const stateLabel: Record<SessionState, string> = {
		idle: 'Idle',
		connecting: 'Connecting',
		running: 'Live',
		reconnecting: 'Reconnecting',
		error: 'Error'
	};

	const stateTone: Record<SessionState, string> = {
		idle: 'neutral',
		connecting: 'warn',
		running: 'live',
		reconnecting: 'warn',
		error: 'bad'
	};

	const MODE_LABEL: Record<OutputMode, string> = {
		translate: 'Translation',
		transcribe: 'Subtitles'
	};
	const SOURCE_LABEL: Record<AudioSource, string> = {
		microphone: 'Room mic',
		system: 'System audio',
		both: 'Both'
	};
	const LANGUAGE_LABEL: Record<TargetLanguage, string> = { en: 'English', fr: 'Français' };
	const ENGINE_LABEL: Record<Provider, string> = {
		gemini: 'Gemini',
		openai: 'OpenAI',
		mistral: 'Voxtral',
		ondevice: 'On-device'
	};
	const COST_NOTE: Record<Provider, string> = {
		gemini:
			'Gemini: input billed on wall clock, output only while it translates — pauses and slide changes lower this.',
		openai: 'OpenAI: audio in and text out are billed per minute for as long as the stream stays open.',
		mistral: 'Voxtral: billed per minute of audio streamed, for as long as the session stays open.',
		ondevice: 'The recognizer runs on this machine, so nothing is billed and no audio leaves it.'
	};
	const ORIGIN_CHIP: Record<Origin, { label: string; sub: string }> = {
		system: { label: 'Remote', sub: 'system' },
		microphone: { label: 'Room', sub: 'mic' }
	};

	// Voxtral detects the spoken language itself, so there is no chosen language to lock.
	const roomReadsLabel = $derived(
		$options.provider === 'mistral' ? 'Auto' : LANGUAGE_LABEL[$options.targetLanguage]
	);

	// Step 03 asks a different question per backend: which language to render into, which one
	// the local recognizer should expect, or none at all when the backend detects it itself.
	const languageStepTitle = $derived(
		$options.mode === 'translate'
			? 'The room reads'
			: $options.provider === 'ondevice'
				? 'Expected language'
				: 'Spoken language'
	);

	/** Split "$1.25–2.21/hr" so the unit can be dimmed; "Free" has no unit to split off. */
	function splitRate(text: string): [string, string] {
		const i = text.indexOf('/');
		return i === -1 ? [text, ''] : [text.slice(0, i), text.slice(i)];
	}

	// The two speakers currently on screen, least-recently-updated first (newest at the bottom).
	const liveTurns = $derived(
		(Object.keys($currentCaptions) as Origin[])
			.map((origin) => ({ origin, caption: $currentCaptions[origin] }))
			.filter((t): t is { origin: Origin; caption: Caption } => t.caption !== undefined)
	);
</script>

<svelte:window
	onkeydown={(e) => {
		if (e.key === 'F2') flipDirection();
	}}
/>

<div class="app">
	<header class="titlebar">
		<span class="brand" aria-hidden="true">
			<svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="#06261b" stroke-width="2.4" stroke-linecap="round"><path d="M4 12.5h3.5L11 6l3 12 2.5-5.5H20" /></svg>
		</span>
		<span class="app-name">Live Captions</span>
		<span class="context">Realtime translation &amp; subtitles</span>
		<span class="grow"></span>
		<div class="pill {stateTone[$sessionState]}" aria-live="polite">
			<span class="pill-dot"></span>
			<span class="pill-label">{stateLabel[$sessionState]}</span>
			{#if $isRunning && $sessionStartedAt !== null}
				<span class="pill-time">{formatElapsed(elapsedMs)}</span>
			{/if}
		</div>
	</header>

	<div class="rule" class:live={$isRunning}>
		{#if $isRunning}<span class="sweep"></span>{/if}
	</div>

	<div class="body">
		<aside class="rail">
			{#if $isRunning}
				<!-- ---- Running: the setup sheet collapses to what it locked in ---- -->
				<div class="rail-head">
					<span class="rail-icon" aria-hidden="true">
						<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><rect x="4.5" y="10.5" width="15" height="10" rx="2.5" /><path d="M8 10.5V8a4 4 0 0 1 8 0v2.5" /></svg>
					</span>
					<span class="kicker">Session locked</span>
				</div>

				<div class="chips">
					<div class="chip">
						<span class="chip-label">Mode</span>
						<span class="chip-value">{MODE_LABEL[$options.mode]}</span>
					</div>
					<div class="chip">
						<span class="chip-label">Source</span>
						<span class="chip-value">{SOURCE_LABEL[$options.source]}</span>
					</div>
					<div class="chip">
						<span class="chip-label">Room reads</span>
						<span class="chip-value">{roomReadsLabel}</span>
					</div>
					<div class="chip">
						<span class="chip-label">Engine</span>
						<span class="chip-value">{ENGINE_LABEL[$options.provider]}</span>
					</div>
				</div>

				<p class="rail-note">
					<!-- The target language is fixed at session start (the backend takes it once),
					     so no mid-session F2 promise here — the idle sheet carries the F2 hint. -->
					<span>Stop the session to change any of these.</span>
				</p>

				<div class="divider"></div>

				<div class="rail-section">
					<span class="kicker">Audio arriving</span>
					{#if usesMic}
						<LevelMeter level={$micLevel} label="Room" active />
					{/if}
					{#if usesSystem}
						<LevelMeter level={$systemLevel} label="System" active />
					{/if}
				</div>

				<div class="cost-card">
					<div class="cost-figures">
						<div class="figure">
							<span class="chip-label">Streamed</span>
							<span class="figure-value">{formatElapsed(elapsedMs)}</span>
						</div>
						{#if $options.provider !== 'ondevice'}
							<div class="figure">
								<span class="chip-label">Est. cost</span>
								<span class="figure-value mint">
									{formatUsd(
										estimateSessionCost($options.provider, elapsedMs, $options.source === 'both' ? 2 : 1)
									)}
								</span>
							</div>
							{#if $options.source === 'both'}
								<span class="cost-tag">×2 sources</span>
							{/if}
						{/if}
					</div>
					<p class="cost-note">{COST_NOTE[$options.provider]}</p>
				</div>

				<div class="divider"></div>

				<div class="rail-section">
					<span class="kicker">Overlay</span>
					<div class="stepper">
						<span class="stepper-label">Caption size</span>
						<button class="step" onclick={() => setFont($overlayFontSize - 2)} aria-label="Smaller captions">−</button>
						<span class="stepper-value">{$overlayFontSize}</span>
						<button class="step" onclick={() => setFont($overlayFontSize + 2)} aria-label="Larger captions">+</button>
					</div>
					<div class="overlay-actions">
						<button class="tool" class:on={moveOverlay} aria-pressed={moveOverlay} onclick={toggleMoveOverlay}>
							<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 3.5v17M3.5 12h17M12 3.5l-3 3M12 3.5l3 3M12 20.5l-3-3M12 20.5l3-3M3.5 12l3-3M3.5 12l3 3M20.5 12l-3-3M20.5 12l-3 3" /></svg>
							{moveOverlay ? 'Done' : 'Move'}
						</button>
						<button class="tool" class:off={!overlayVisible} onclick={toggleOverlayVisible}>
							<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" aria-hidden="true">
								<rect x="2.5" y="4.5" width="19" height="13" rx="2" /><path d="M9 20.5h6" />
								{#if overlayVisible}<path d="M3.5 20.5l17-17" />{/if}
							</svg>
							{overlayVisible ? 'Hide' : 'Show'}
						</button>
					</div>
				</div>

				<span class="grow"></span>

				<button class="stop" disabled={sessionBusy} onclick={stop}>
					<svg width="13" height="13" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><rect x="6" y="6" width="12" height="12" rx="2" /></svg>
					{sessionBusy ? 'Stopping…' : 'Stop captions'}
				</button>
			{:else}
				<!-- ---- Idle: the numbered setup sheet ---- -->
				<section class="rail-section">
					<div class="step-head">
						<span class="step-no">01</span>
						<span class="kicker">What to show</span>
					</div>
					<button
						class="card"
						class:selected={$options.mode === 'translate'}
						disabled={controlsLocked}
						aria-pressed={$options.mode === 'translate'}
						onclick={() => setMode('translate')}
					>
						<span class="card-icon" aria-hidden="true">
							<svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"><path d="M4 8.5h13l-3.5-3.5" /><path d="M20 15.5H7l3.5 3.5" /></svg>
						</span>
						<span class="card-body">
							<span class="card-title">Live translation</span>
							<span class="card-desc">Speech is detected and translated into the language the room reads.</span>
						</span>
						{#if $options.mode === 'translate'}
							<span class="card-check" aria-hidden="true">
								<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M4 12.5l5 5L20 6.5" /></svg>
							</span>
						{/if}
					</button>
					<button
						class="card"
						class:selected={$options.mode === 'transcribe'}
						disabled={controlsLocked}
						aria-pressed={$options.mode === 'transcribe'}
						onclick={() => setMode('transcribe')}
					>
						<span class="card-icon" aria-hidden="true">
							<svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"><path d="M4 7h16M4 12h11M4 17h7" /></svg>
						</span>
						<span class="card-body">
							<span class="card-title">Live subtitles</span>
							<span class="card-desc">Same-language text, no translation. Saveable as text or Markdown.</span>
						</span>
						{#if $options.mode === 'transcribe'}
							<span class="card-check" aria-hidden="true">
								<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M4 12.5l5 5L20 6.5" /></svg>
							</span>
						{/if}
					</button>
				</section>

				<div class="divider"></div>

				<section class="rail-section">
					<div class="step-head">
						<span class="step-no">02</span>
						<span class="kicker">Where the audio comes from</span>
					</div>
					<div class="tiles">
						<button
							class="tile"
							class:selected={$options.source === 'microphone'}
							disabled={controlsLocked}
							aria-pressed={$options.source === 'microphone'}
							onclick={() => setSource('microphone')}
						>
							<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" aria-hidden="true"><rect x="9" y="2.5" width="6" height="11" rx="3" /><path d="M5.5 11.5a6.5 6.5 0 0 0 13 0" /><path d="M12 18v3.5" /></svg>
							<span>Room mic</span>
						</button>
						<button
							class="tile"
							class:selected={$options.source === 'system'}
							disabled={controlsLocked}
							aria-pressed={$options.source === 'system'}
							onclick={() => setSource('system')}
						>
							<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M4 9.5h3.5L13 5v14L7.5 14.5H4z" /><path d="M16.5 9.2a4.2 4.2 0 0 1 0 5.6" /></svg>
							<span>System audio</span>
						</button>
						<button
							class="tile"
							class:selected={$options.source === 'both'}
							disabled={controlsLocked}
							aria-pressed={$options.source === 'both'}
							onclick={() => setSource('both')}
						>
							<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" aria-hidden="true"><path d="M3.5 6.5h4.5L12 12l4 5.5h4.5M3.5 17.5h4.5L12 12" /></svg>
							<span>Both</span>
						</button>
					</div>
					<p class="hint">
						System audio captures whatever is playing on this machine — Zoom, Teams, a browser tab, a
						media player.
					</p>

					{#if usesMic}
						<div class="select-row">
							<select
								aria-label="Microphone device"
								disabled={controlsLocked}
								value={$options.micDeviceName ?? ''}
								onchange={(e) => ($options = { ...$options, micDeviceName: e.currentTarget.value || null })}
							>
								<option value="">System default</option>
								{#each microphones as dev (dev.name)}
									<option value={dev.name}>{dev.name}{dev.isDefault ? ' (default)' : ''}</option>
								{/each}
							</select>
							<svg class="chevron" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><path d="M6 9.5l6 6 6-6" /></svg>
						</div>
					{/if}

					<div class="meters">
						{#if usesMic}
							<LevelMeter level={$micLevel} label="Room mic" />
						{/if}
						{#if usesSystem}
							<LevelMeter level={$systemLevel} label="System" />
						{/if}
					</div>
				</section>

				<div class="divider"></div>

				<section class="rail-section">
					<div class="step-head">
						<span class="step-no">03</span>
						<span class="kicker">{languageStepTitle}</span>
					</div>
					{#if $options.mode === 'transcribe' && $options.provider === 'mistral'}
						<p class="hint">
							Voxtral auto-detects the spoken language and writes same-language subtitles. No
							translation target is needed.
						</p>
					{:else}
						<div class="lang-cards">
							<button
								class="lang"
								class:selected={$options.targetLanguage === 'en'}
								disabled={controlsLocked}
								aria-pressed={$options.targetLanguage === 'en'}
								onclick={() => setTarget('en')}
							>
								<span class="lang-code">EN</span>
								<span class="lang-name">English</span>
							</button>
							<button
								class="lang"
								class:selected={$options.targetLanguage === 'fr'}
								disabled={controlsLocked}
								aria-pressed={$options.targetLanguage === 'fr'}
								onclick={() => setTarget('fr')}
							>
								<span class="lang-code">FR</span>
								<span class="lang-name">Français</span>
							</button>
						</div>
						{#if $options.mode === 'translate'}
							<p class="hint inline-hint">
								<span>Speakers alternating? Flip mid-session with</span>
								<span class="key">F2</span>
							</p>
						{:else}
							<p class="hint">
								The on-device recognizer is told which language to expect, so pick the one that will be
								spoken. It writes same-language subtitles and never translates.
							</p>
						{/if}
					{/if}
				</section>

				<div class="divider"></div>

				<section class="rail-section">
					<div class="step-head">
						<span class="step-no">04</span>
						<span class="kicker">Engine</span>
					</div>
					<div class="engines">
						{#each modeProviders as id (id)}
							{@const p = PROVIDER_META[id]}
							{@const rate = splitRate(p.hourlyText)}
							<button
								class="engine"
								class:selected={$options.provider === id}
								disabled={controlsLocked}
								aria-pressed={$options.provider === id}
								onclick={() => setProvider(id)}
							>
								<span class="engine-body">
									<span class="engine-name">{p.vendor}</span>
									<span class="engine-model">{p.modelId}</span>
								</span>
								<span class="engine-rate">{rate[0]}<span class="unit">{rate[1]}</span></span>
							</button>
						{/each}
					</div>
				</section>
			{/if}
		</aside>

		<main class="stage">
			{#if browserMode}
				<div class="banner">
					Running in a browser without the Tauri runtime — controls are disabled. Launch with
					<code>npm run tauri dev</code> for audio capture, translation, and subtitles.
				</div>
			{/if}

			{#if $isRunning}
				<div class="stage-head">
					<span class="kicker">On screen now</span>
					<span class="stretch"></span>
					<span class="stage-note">
						{liveTurns.length > 1 ? 'Two speakers · newest at the bottom' : 'Newest at the bottom'}
					</span>
				</div>

				{#if liveTurns.length}
					<div class="turns">
						{#each liveTurns as turn (turn.origin)}
							<article class="turn">
								<div class="turn-who">
									<span class="origin-chip {turn.origin}">{ORIGIN_CHIP[turn.origin].label}</span>
									<span class="origin-sub">{ORIGIN_CHIP[turn.origin].sub}</span>
								</div>
								<div class="turn-text">
									{#if turn.caption.sourceText}
										<p class="turn-source">{turn.caption.sourceText}</p>
									{/if}
									<p
										class="turn-caption"
										class:live={!turn.caption.final}
									>{turn.caption.text}{#if !turn.caption.final}<span class="caret"></span>{/if}</p>
								</div>
							</article>
						{/each}
					</div>
				{:else}
					<p class="hint stage-hint">
						{$options.mode === 'translate'
							? 'Translated captions will appear here and on the overlay.'
							: 'Live subtitles will appear here and on the overlay.'}
					</p>
				{/if}

				{#if $statusMessage}
					<p class="status-msg">{$statusMessage}</p>
				{/if}

				<span class="grow"></span>

				<TranscriptMonitor
					mode={$options.mode}
					transcript={$transcript}
					onError={(message) => statusMessage.set(message)}
				/>
			{:else}
				<span class="kicker">Pre-flight</span>
				<h1>Ready when you are</h1>
				<p class="intro">
					Four checks, then one button. Everything on the left locks while captions are running, so
					nothing can be changed by accident mid-session.
				</p>

				<div class="checklist">
					{#if !needsKey}
						<div class="check-row">
							<span class="mark ok">
								<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" aria-hidden="true"><path d="M4 12.5l5 5L20 6.5" /></svg>
							</span>
							<div class="check-body">
								<span class="check-title">No key needed</span>
								<span class="check-desc">Runs entirely on this machine</span>
							</div>
							<span></span>
						</div>
					{:else if !browserMode}
						<ApiKeyPanel
							provider={$options.provider}
							locked={controlsLocked}
							onAvailability={(provider, available) => {
								if ($options.provider === provider) $hasKey = available;
							}}
							onError={(message) => statusMessage.set(message)}
						/>
					{/if}

					<div class="check-row">
						{#if audioReady}
							<span class="mark ok">
								<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" aria-hidden="true"><path d="M4 12.5l5 5L20 6.5" /></svg>
							</span>
						{:else}
							<span class="mark wait"><span class="dot"></span></span>
						{/if}
						<div class="check-body">
							<span class="check-title">{audioTitle}</span>
							<span class="check-desc" class:warn={!audioReady}>
								{audioReady ? audioReadyDesc : 'Waiting for sound — play or say something'}
							</span>
						</div>
						<span></span>
					</div>

					<div class="check-row">
						{#if $overlayPlaced}
							<span class="mark ok">
								<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" aria-hidden="true"><path d="M4 12.5l5 5L20 6.5" /></svg>
							</span>
						{:else}
							<span class="mark wait"><span class="dot"></span></span>
						{/if}
						<div class="check-body">
							<span class="check-title">Overlay placement</span>
							<span class="check-desc" class:warn={!$overlayPlaced}>
								{$overlayPlaced
									? 'Placed — captions will appear where you locked them'
									: 'Not placed yet — captions will sit bottom-centre on this display'}
							</span>
						</div>
						<!-- Placement is never final: re-entering move mode is the way to adjust
						     position and caption size, so the row keeps a button in both states. -->
						{#if $overlayPlaced}
							<button class="adjust" aria-pressed={moveOverlay} onclick={toggleMoveOverlay}>
								{moveOverlay ? 'Done' : 'Adjust'}
							</button>
						{:else}
							<button class="place" aria-pressed={moveOverlay} onclick={toggleMoveOverlay}>
								{moveOverlay ? 'Done' : 'Place it'}
							</button>
						{/if}
					</div>

					<div class="check-row">
						<span class="mark neutral">$</span>
						<div class="check-body">
							<span class="check-title">Running cost</span>
							<span class="check-desc">
								{$options.provider === 'ondevice'
									? 'Runs locally — nothing is billed'
									: 'Billed per minute of streamed audio, for as long as the session is open'}
							</span>
						</div>
						<span class="check-rate">{meta.hourlyText}</span>
					</div>
				</div>

				<!-- Straight after Stop, saving the transcript is the operator's next job, so it
				     sits above the Start row — below it, past the spacer, it scrolls out of view
				     and reads as lost (the store keeps the lines regardless). -->
				{#if $transcript.length > 0}
					<TranscriptMonitor
						mode={$options.mode}
						transcript={$transcript}
						onError={(message) => statusMessage.set(message)}
					/>
				{/if}

				<span class="grow"></span>

				{#if $statusMessage}
					<p class="status-msg">{$statusMessage}</p>
				{/if}

				<div class="launch">
					<button class="start" disabled={!$hasKey || browserMode || sessionBusy} onclick={start}>
						<svg width="15" height="15" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M8 5.5l11 6.5-11 6.5z" /></svg>
						{sessionBusy
							? 'Starting…'
							: $options.mode === 'translate'
								? 'Start translating'
								: 'Start subtitles'}
					</button>
					<span class="privacy">
						Transcript is held in memory until you save it.
						{$options.provider === 'ondevice'
							? 'Nothing leaves the machine at all.'
							: `Nothing leaves the machine except audio to ${meta.vendor}.`}
					</span>
				</div>
			{/if}
		</main>
	</div>
</div>

<style>
	.app {
		height: 100vh;
		display: grid;
		grid-template-rows: 40px 2px minmax(0, 1fr);
		background: var(--surface-0);
	}

	/* ---- Header ------------------------------------------------------------- */

	.titlebar {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 0 14px;
		background: #12151a;
		border-bottom: 1px solid var(--hairline);
	}
	.brand {
		width: 18px;
		height: 18px;
		border-radius: 5px;
		background: linear-gradient(150deg, #5ad1a0, #2f8f6b);
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 0 0 auto;
	}
	.app-name {
		font-size: 12.5px;
		font-weight: 500;
		line-height: 1;
	}
	.context {
		font-size: 12px;
		line-height: 1;
		color: var(--muted-3);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.grow {
		flex: 1;
	}
	.pill {
		display: flex;
		align-items: center;
		gap: 7px;
		padding: 4px 10px 4px 8px;
		border-radius: 20px;
		flex: 0 0 auto;
	}
	.pill-dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
	}
	.pill-label {
		font-size: 11px;
		font-weight: 500;
		line-height: 1;
		letter-spacing: 0.06em;
		text-transform: uppercase;
	}
	.pill-time {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 500;
		line-height: 1;
		font-variant-numeric: tabular-nums;
	}
	.pill.neutral {
		background: var(--surface-3);
		border: 1px solid var(--border);
	}
	.pill.neutral .pill-dot {
		background: var(--faint);
	}
	.pill.neutral .pill-label {
		color: var(--muted);
	}
	.pill.live {
		background: var(--accent-bg);
		border: 1px solid var(--accent-border);
	}
	.pill.live .pill-dot {
		background: var(--accent);
		box-shadow: 0 0 8px var(--accent);
		animation: breathe 2.4s ease-in-out infinite;
	}
	.pill.live .pill-label,
	.pill.live .pill-time {
		color: var(--accent-soft);
		font-weight: 600;
	}
	.pill.warn {
		background: var(--warn-bg);
		border: 1px solid var(--warn-border);
	}
	.pill.warn .pill-dot {
		background: var(--warn);
		animation: breathe 2.4s ease-in-out infinite;
	}
	.pill.warn .pill-label {
		color: var(--warn-soft);
	}
	.pill.bad {
		background: var(--danger-bg);
		border: 1px solid var(--danger-border);
	}
	.pill.bad .pill-dot {
		background: var(--danger);
	}
	.pill.bad .pill-label {
		color: var(--danger-soft);
	}

	.rule {
		background: var(--hairline);
	}
	.rule.live {
		background: #163027;
		position: relative;
		overflow: hidden;
	}
	.sweep {
		position: absolute;
		inset: 0;
		background: linear-gradient(90deg, transparent, var(--accent), transparent);
		animation: sweep 2.6s linear infinite;
	}

	/* ---- Layout ------------------------------------------------------------- */

	.body {
		display: grid;
		grid-template-columns: 380px minmax(0, 1fr);
		min-height: 0;
	}
	.rail {
		padding: 22px 22px 26px;
		display: flex;
		flex-direction: column;
		gap: 20px;
		background: var(--panel);
		border-right: 1px solid var(--hairline);
		overflow-y: auto;
	}
	.stage {
		padding: 30px 38px 32px;
		display: flex;
		flex-direction: column;
		overflow-y: auto;
	}
	.divider {
		height: 1px;
		background: var(--hairline);
	}
	/* Both columns scroll rather than compress: a flex column shrinks its children before the
	   scrollbar appears, which would clip text on a short window. */
	.divider,
	.kicker,
	.rail-head,
	.rail-section,
	.chips,
	.rail-note,
	.cost-card,
	.stop,
	.banner,
	h1,
	.intro,
	.checklist,
	.status-msg,
	.launch,
	.stage-head,
	.turns,
	.stage-hint {
		flex: 0 0 auto;
	}

	/* ---- Rail shared -------------------------------------------------------- */

	.rail-section {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.step-head,
	.rail-head {
		display: flex;
		align-items: center;
		gap: 9px;
	}
	.rail-icon {
		color: var(--muted-3);
		display: flex;
	}
	.step-no {
		font-family: var(--font-mono);
		font-size: 10.5px;
		font-weight: 500;
		line-height: 1;
		color: var(--accent);
	}
	.kicker {
		font-size: 10.5px;
		font-weight: 600;
		line-height: 1;
		letter-spacing: 0.15em;
		text-transform: uppercase;
		color: var(--muted-2);
	}
	.hint {
		margin: 0;
		font-size: 11.5px;
		line-height: 1.5;
		color: var(--muted-2);
	}
	.inline-hint {
		display: flex;
		align-items: center;
		gap: 8px;
		line-height: 1.4;
	}
	.key {
		font-family: var(--font-mono);
		font-size: 10.5px;
		font-weight: 500;
		line-height: 1;
		color: var(--text-dim);
		padding: 4px 6px;
		border-radius: 5px;
		border: 1px solid #2a2f38;
		background: var(--surface-3);
		white-space: nowrap;
	}

	/* ---- Selection cards ---------------------------------------------------- */

	.card,
	.tile,
	.lang,
	.engine {
		border: 1px solid var(--border);
		background: var(--panel-2);
		text-align: left;
		color: inherit;
	}
	.card.selected,
	.tile.selected,
	.lang.selected,
	.engine.selected {
		border-color: var(--accent-border);
		background: var(--accent-bg);
	}
	.card:hover:not(:disabled),
	.tile:hover:not(:disabled),
	.lang:hover:not(:disabled),
	.engine:hover:not(:disabled) {
		border-color: var(--border-hover);
	}
	.card.selected:hover:not(:disabled),
	.tile.selected:hover:not(:disabled),
	.lang.selected:hover:not(:disabled),
	.engine.selected:hover:not(:disabled) {
		border-color: var(--accent);
	}

	.card {
		display: flex;
		align-items: flex-start;
		gap: 11px;
		padding: 12px 13px;
		border-radius: 11px;
		width: 100%;
	}
	.card-icon {
		width: 30px;
		height: 30px;
		border-radius: 8px;
		background: rgba(255, 255, 255, 0.045);
		color: var(--muted);
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 0 0 auto;
	}
	.card.selected .card-icon {
		background: var(--accent-chip-bg);
		color: var(--accent-soft);
	}
	.card-body {
		display: flex;
		flex-direction: column;
		gap: 3px;
		min-width: 0;
	}
	.card-title {
		font-size: 13.5px;
		font-weight: 600;
		line-height: 1.2;
		color: var(--text-soft);
	}
	.card.selected .card-title {
		color: var(--text);
	}
	.card-desc {
		font-size: 11.5px;
		line-height: 1.45;
		color: var(--muted-2);
		text-wrap: pretty;
	}
	.card.selected .card-desc {
		color: var(--muted);
	}
	.card-check {
		color: var(--accent);
		margin-left: auto;
		flex: 0 0 auto;
		display: flex;
	}

	.tiles {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 8px;
	}
	.tile {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 7px;
		padding: 12px 6px 10px;
		border-radius: 10px;
		color: var(--muted);
		font-size: 11.5px;
		font-weight: 500;
		line-height: 1;
	}
	.tile.selected {
		color: var(--accent-soft);
		font-weight: 600;
	}

	.select-row {
		position: relative;
		display: flex;
		align-items: center;
		padding: 10px 12px;
		border-radius: 9px;
		border: 1px solid var(--border);
		background: var(--panel-2);
		margin-top: 2px;
	}
	.select-row select {
		appearance: none;
		width: 100%;
		border: 0;
		background: transparent;
		color: var(--text-soft);
		font-size: 12.5px;
		line-height: 1;
		padding: 0 20px 0 0;
	}
	.select-row select:focus-visible {
		outline: 2px solid var(--accent-border);
		outline-offset: 4px;
		border-radius: 3px;
	}
	.select-row select option {
		background: var(--panel-2);
		color: var(--text);
	}
	.chevron {
		position: absolute;
		right: 12px;
		color: var(--muted-2);
		pointer-events: none;
	}
	.meters {
		display: flex;
		flex-direction: column;
		gap: 9px;
		margin-top: 6px;
	}

	.lang-cards {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 8px;
	}
	.lang {
		display: flex;
		align-items: center;
		gap: 9px;
		padding: 11px 12px;
		border-radius: 10px;
	}
	.lang-code {
		font-family: var(--font-mono);
		font-size: 12px;
		font-weight: 500;
		line-height: 1;
		color: var(--muted-2);
	}
	.lang.selected .lang-code {
		color: var(--accent-soft);
		font-weight: 600;
	}
	.lang-name {
		font-size: 12.5px;
		font-weight: 500;
		line-height: 1;
		color: var(--text-soft);
	}
	.lang.selected .lang-name {
		color: var(--text);
		font-weight: 600;
	}

	.engines {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.engine {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 11px 13px;
		border-radius: 10px;
	}
	.engine-body {
		display: flex;
		flex-direction: column;
		gap: 3px;
		min-width: 0;
	}
	.engine-name {
		font-size: 12.5px;
		font-weight: 500;
		line-height: 1;
		color: var(--text-soft);
	}
	.engine.selected .engine-name {
		color: var(--text);
		font-weight: 600;
	}
	.engine-model {
		font-family: var(--font-mono);
		font-size: 10.5px;
		line-height: 1.2;
		color: var(--muted-3);
		overflow-wrap: anywhere;
	}
	.engine.selected .engine-model {
		color: var(--muted-2);
	}
	.engine-rate {
		margin-left: auto;
		font-family: var(--font-mono);
		font-size: 11.5px;
		font-weight: 500;
		line-height: 1;
		color: var(--muted);
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}
	.engine.selected .engine-rate {
		color: var(--accent-soft);
	}
	.engine-rate .unit {
		color: var(--muted-3);
	}

	/* ---- Rail, running ------------------------------------------------------ */

	.chips {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 8px;
	}
	.chip {
		padding: 9px 11px;
		border-radius: 9px;
		background: var(--panel-2);
		border: 1px solid var(--border-2);
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
	}
	.chip-label {
		font-size: 9.5px;
		font-weight: 500;
		line-height: 1;
		letter-spacing: 0.12em;
		text-transform: uppercase;
		color: var(--muted-3);
	}
	.chip-value {
		font-size: 12.5px;
		font-weight: 500;
		line-height: 1.1;
		color: #dfe3e9;
	}
	.rail-note {
		margin: 0;
		font-size: 11.5px;
		line-height: 1.4;
		color: var(--muted-3);
	}

	.cost-card {
		padding: 14px 15px;
		border-radius: 11px;
		background: var(--panel-2);
		border: 1px solid var(--border-2);
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.cost-figures {
		display: flex;
		align-items: baseline;
		gap: 18px;
	}
	.figure {
		display: flex;
		flex-direction: column;
		gap: 5px;
	}
	.figure-value {
		font-family: var(--font-mono);
		font-size: 21px;
		font-weight: 500;
		line-height: 1;
		color: #dfe3e9;
		font-variant-numeric: tabular-nums;
	}
	.figure-value.mint {
		color: var(--accent-soft);
	}
	.cost-tag {
		margin-left: auto;
		font-family: var(--font-mono);
		font-size: 10.5px;
		font-weight: 500;
		line-height: 1;
		color: var(--muted-3);
	}
	.cost-note {
		margin: 0;
		font-size: 11px;
		line-height: 1.45;
		color: var(--muted-3);
		text-wrap: pretty;
	}

	.stepper {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.stepper-label {
		font-size: 12px;
		line-height: 1;
		color: var(--muted);
		flex: 1;
	}
	.stepper-value {
		font-family: var(--font-mono);
		font-size: 13px;
		font-weight: 500;
		line-height: 1;
		min-width: 26px;
		text-align: center;
		font-variant-numeric: tabular-nums;
	}
	.step {
		width: 30px;
		height: 30px;
		border-radius: 8px;
		border: 1px solid var(--border);
		background: var(--panel-2);
		color: var(--text-soft);
		font-size: 14px;
		font-weight: 500;
		line-height: 1;
	}
	.step:hover {
		border-color: var(--border-hover);
	}

	.overlay-actions {
		display: flex;
		gap: 8px;
	}
	.tool {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 7px;
		padding: 10px;
		border-radius: 9px;
		border: 1px solid var(--border);
		background: var(--panel-2);
		color: var(--text-soft);
		font-size: 12px;
		font-weight: 500;
		line-height: 1;
	}
	.tool:hover {
		border-color: var(--border-hover);
		color: var(--text);
	}
	.tool.on {
		border-color: var(--accent-border);
		background: var(--accent-bg);
		color: var(--accent-soft);
	}
	/* Captions are blanked — the amber tint says the room is currently seeing nothing. */
	.tool.off {
		border-color: var(--warn-border);
		background: rgba(255, 180, 84, 0.08);
		color: var(--warn-soft);
	}

	.stop {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 10px;
		padding: 14px;
		border-radius: 11px;
		border: 1px solid var(--danger-border);
		background: var(--danger-bg);
		color: var(--danger-soft);
		font-size: 14px;
		font-weight: 600;
		line-height: 1;
	}
	.stop:hover:not(:disabled) {
		background: rgba(255, 92, 92, 0.18);
		color: #ffb3b3;
	}

	/* ---- Stage -------------------------------------------------------------- */

	.banner {
		background: var(--panel-2);
		border: 1px solid var(--border);
		border-radius: 10px;
		padding: 12px;
		margin-bottom: 22px;
		font-size: 13px;
		line-height: 1.5;
		color: var(--muted);
	}
	.banner code {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-dim);
	}
	h1 {
		margin: 16px 0 0;
		font-size: 27px;
		font-weight: 600;
		line-height: 1.2;
		letter-spacing: -0.02em;
	}
	.intro {
		margin: 8px 0 0;
		font-size: 13.5px;
		line-height: 1.55;
		color: var(--muted);
		max-width: 48ch;
		text-wrap: pretty;
	}

	.checklist {
		margin-top: 26px;
		border-top: 1px solid var(--hairline);
	}
	.check-row {
		display: grid;
		grid-template-columns: 24px 1fr auto;
		align-items: center;
		gap: 14px;
		padding: 15px 0;
		border-bottom: 1px solid var(--hairline);
	}
	.mark {
		width: 20px;
		height: 20px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.mark.ok {
		background: var(--accent-chip-bg);
		color: var(--accent);
	}
	.mark.wait {
		background: var(--warn-bg);
		color: var(--warn);
	}
	.mark.wait .dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: currentColor;
	}
	.mark.neutral {
		background: rgba(255, 255, 255, 0.05);
		color: var(--muted);
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 500;
		line-height: 1;
	}
	.check-body {
		display: flex;
		flex-direction: column;
		gap: 3px;
		min-width: 0;
	}
	.check-title {
		font-size: 13.5px;
		font-weight: 500;
		line-height: 1.2;
	}
	.check-desc {
		font-size: 12px;
		line-height: 1.3;
		color: var(--muted-2);
	}
	.check-desc.warn {
		color: var(--warn);
	}
	.check-rate {
		font-family: var(--font-mono);
		font-size: 12.5px;
		font-weight: 500;
		line-height: 1;
		color: var(--text-soft);
		font-variant-numeric: tabular-nums;
	}
	.place {
		font-size: 11.5px;
		font-weight: 500;
		line-height: 1;
		color: var(--warn-soft);
		padding: 7px 11px;
		border-radius: 7px;
		border: 1px solid var(--warn-border);
		background: rgba(255, 180, 84, 0.08);
	}
	.place:hover {
		background: var(--warn-bg);
	}
	/* Quiet variant of .place for the already-placed row: same geometry, ghost colours. */
	.adjust {
		font-size: 11.5px;
		font-weight: 500;
		line-height: 1;
		color: var(--text-soft);
		padding: 7px 11px;
		border-radius: 7px;
		border: 1px solid var(--border);
		background: transparent;
	}
	.adjust:hover {
		border-color: var(--border-hover);
		color: var(--text);
	}

	.launch {
		display: flex;
		align-items: center;
		gap: 16px;
		margin-top: 30px;
	}
	.start {
		display: flex;
		align-items: center;
		gap: 11px;
		padding: 15px 24px;
		border: 0;
		border-radius: 12px;
		background: linear-gradient(#5ad1a0, #43b989);
		color: var(--on-accent);
		font-size: 15.5px;
		font-weight: 600;
		line-height: 1;
		box-shadow: 0 12px 30px -12px rgba(90, 209, 160, 0.65);
		flex: 0 0 auto;
	}
	.start:hover:not(:disabled) {
		filter: brightness(1.06);
	}
	.start:disabled {
		box-shadow: none;
	}
	.privacy {
		font-size: 12.5px;
		line-height: 1.5;
		color: var(--muted-3);
		max-width: 34ch;
		text-wrap: pretty;
	}
	.status-msg {
		margin: 16px 0 0;
		font-size: 13px;
		line-height: 1.5;
		color: var(--warn);
	}

	.stage-head {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.stretch {
		flex: 1;
		height: 1px;
		background: var(--hairline);
	}
	.stage-note {
		font-size: 11.5px;
		line-height: 1;
		color: var(--muted-3);
	}
	.stage-hint {
		margin-top: 24px;
		font-size: 13.5px;
	}
	.turns {
		display: flex;
		flex-direction: column;
		margin-top: 8px;
	}
	.turn {
		display: grid;
		grid-template-columns: 96px minmax(0, 1fr);
		gap: 20px;
		padding: 24px 0;
		border-bottom: 1px solid var(--hairline);
	}
	.turn-who {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding-top: 4px;
	}
	.origin-chip {
		align-self: flex-start;
		font-size: 10px;
		font-weight: 600;
		line-height: 1;
		letter-spacing: 0.14em;
		text-transform: uppercase;
		padding: 5px 8px;
		border-radius: 5px;
	}
	.origin-chip.system {
		color: var(--accent-soft);
		background: var(--accent-chip-bg);
	}
	.origin-chip.microphone {
		color: var(--room-soft);
		background: var(--room-bg);
	}
	.origin-sub {
		font-family: var(--font-mono);
		font-size: 10.5px;
		line-height: 1;
		color: var(--muted-3);
	}
	.turn-text {
		display: flex;
		flex-direction: column;
		gap: 10px;
		max-width: 60ch;
	}
	.turn-source {
		margin: 0;
		font-size: 13px;
		line-height: 1.5;
		color: var(--muted-2);
		text-wrap: pretty;
	}
	.turn-caption {
		margin: 0;
		font-size: 29px;
		font-weight: 600;
		line-height: 1.3;
		letter-spacing: -0.015em;
		color: var(--text-dim);
		text-wrap: pretty;
	}
	.turn-caption.live {
		color: var(--text-bright);
	}
	.caret {
		display: inline-block;
		width: 3px;
		height: 0.9em;
		background: var(--accent);
		margin-left: 6px;
		vertical-align: -2px;
		animation: blink 1.1s steps(1) infinite;
	}

	/* Near the window's minimum height, tighten the vertical rhythm so the pre-flight checklist
	   and the Start button still land above the fold. */
	@media (max-height: 740px) {
		.rail {
			padding: 18px 22px 20px;
			gap: 16px;
		}
		.stage {
			padding-top: 22px;
			padding-bottom: 22px;
		}
		h1 {
			margin-top: 10px;
			font-size: 24px;
		}
		.checklist {
			margin-top: 18px;
		}
		.check-row {
			padding: 12px 0;
		}
		.launch {
			margin-top: 22px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.sweep,
		.pill-dot,
		.caret {
			animation: none;
		}
	}
</style>
