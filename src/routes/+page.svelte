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
		latestCaption,
		transcript,
		micLevel,
		systemLevel,
		overlayFontSize,
		pushCaption,
		beginSession
	} from '$lib/stores';
	import type {
		AudioDevice,
		AudioSource,
		OutputMode,
		Provider,
		TargetLanguage
	} from '$lib/types';
	import { clampOverlayFont } from '$lib/types';
	import LevelMeter from '$lib/LevelMeter.svelte';
	import ApiKeyPanel from '$lib/ApiKeyPanel.svelte';
	import TranscriptMonitor from '$lib/TranscriptMonitor.svelte';

	let microphones = $state<AudioDevice[]>([]);
	let browserMode = $state(false);
	let sessionBusy = $state(false);
	const controlsLocked = $derived($isRunning || sessionBusy);

	onMount(() => {
		browserMode = !isTauri();
		if (browserMode) return;

		void refresh();
		// Sync the overlay to the operator's current caption size on load.
		void api.setOverlayConfig({ fontSize: $overlayFontSize });

		const unlisteners: Array<Promise<() => void>> = [
			on.caption((c) => pushCaption(c)),
			on.level((l) => (l.source === 'microphone' ? micLevel.set(l) : systemLevel.set(l))),
			on.status((s) => applyStatus(s))
		];

		return () => {
			void Promise.all(unlisteners).then((fns) => fns.forEach((f) => f()));
		};
	});

	async function refresh() {
		try {
			microphones = await api.listMicrophones();
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
		if (controlsLocked || $options.mode !== 'translate') return;
		$options = { ...$options, targetLanguage: t };
	}

	function setProvider(p: Provider) {
		if (controlsLocked || p === 'mistral') return;
		if (p === $options.provider) return;
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
	let moveOverlay = $state(false);

	async function toggleMoveOverlay() {
		moveOverlay = !moveOverlay;
		try {
			await api.showOverlay(true);
			await api.setOverlayClickThrough(!moveOverlay);
			await api.setOverlayConfig({ fontSize: $overlayFontSize, interactive: moveOverlay });
		} catch (e) {
			statusMessage.set(String(e));
		}
	}

	const stateLabel: Record<string, string> = {
		idle: 'Idle',
		connecting: 'Connecting…',
		running: 'Live',
		reconnecting: 'Reconnecting…',
		error: 'Error'
	};
</script>

<svelte:window
	onkeydown={(e) => {
		if (e.key === 'F2') flipDirection();
	}}
/>

<main>
	<header>
		<div class="title">
			<h1>Live Captions</h1>
			<span class="subtitle">Translation · subtitles · STIAS DH &amp; AI workshop</span>
		</div>
		<div class="state state-{$sessionState}">
			<span class="dot"></span>
			{stateLabel[$sessionState] ?? $sessionState}
		</div>
	</header>

	{#if browserMode}
		<div class="banner">
			Running in a browser without the Tauri runtime — controls are disabled. Launch with
			<code>npm run tauri dev</code> for audio capture, translation, and subtitles.
		</div>
	{/if}

	<section class="panel">
		<h2>Caption mode</h2>
		<div class="segmented mode-select">
			<button
				class:active={$options.mode === 'translate'}
				disabled={controlsLocked}
				onclick={() => setMode('translate')}
			>
				Live translation
			</button>
			<button
				class:active={$options.mode === 'transcribe'}
				disabled={controlsLocked}
				onclick={() => setMode('transcribe')}
			>
				Live subtitles
			</button>
		</div>
		<p class="hint engine-hint">
			{#if $options.mode === 'translate'}
				Translate speech into English or French using Gemini or OpenAI.
			{:else}
				Transcribe speech in its original language with Mistral Voxtral; save the result as plain text or Markdown.
			{/if}
		</p>
	</section>

	{#if !browserMode}
		<ApiKeyPanel
			provider={$options.provider}
			locked={controlsLocked}
			onAvailability={(provider, available) => {
				if ($options.provider === provider) $hasKey = available;
			}}
			onError={(message) => statusMessage.set(message)}
		/>
	{/if}

	<div class="grid">
		<section class="panel">
			<h2>Audio source</h2>
			<p class="hint">
				System audio captures whatever is playing on this machine — Zoom, Teams, a browser tab,
				a media player.
			</p>
			<div class="segmented">
				<button disabled={controlsLocked} class:active={$options.source === 'microphone'} onclick={() => setSource('microphone')}>
					🎤 Microphone
				</button>
				<button disabled={controlsLocked} class:active={$options.source === 'system'} onclick={() => setSource('system')}>
					🔊 System audio
				</button>
				<button disabled={controlsLocked} class:active={$options.source === 'both'} onclick={() => setSource('both')}>
					Both
				</button>
			</div>

			{#if $options.source !== 'system'}
				<label class="field">
					<span>Microphone device</span>
					<select
						disabled={controlsLocked}
						value={$options.micDeviceName ?? ''}
						onchange={(e) =>
							($options = { ...$options, micDeviceName: e.currentTarget.value || null })}
					>
						<option value="">System default</option>
						{#each microphones as dev (dev.name)}
							<option value={dev.name}>{dev.name}{dev.isDefault ? ' (default)' : ''}</option>
						{/each}
					</select>
				</label>
			{/if}

			<div class="meters">
				{#if $options.source !== 'system'}
					<LevelMeter level={$micLevel} label="Mic" />
				{/if}
				{#if $options.source !== 'microphone'}
					<LevelMeter level={$systemLevel} label="System" />
				{/if}
			</div>
		</section>

		<section class="panel">
			{#if $options.mode === 'translate'}
				<h2>Caption language</h2>
				<p class="hint">
					Spoken language is auto-detected; pick the language the audience reads. Speech already
					in that language remains captioned.
				</p>
				<div class="segmented">
					<button disabled={controlsLocked} class:active={$options.targetLanguage === 'en'} onclick={() => setTarget('en')}>
						🇬🇧 English
					</button>
					<button disabled={controlsLocked} class:active={$options.targetLanguage === 'fr'} onclick={() => setTarget('fr')}>
						🇫🇷 Français
					</button>
				</div>
				<button class="ghost flip" disabled={controlsLocked} onclick={flipDirection}>⇄ Flip (F2)</button>
			{:else}
				<h2>Subtitle language</h2>
				<p class="hint">
					Voxtral auto-detects the spoken language and writes same-language subtitles. No translation target is needed.
				</p>
			{/if}
		</section>
	</div>

	<section class="panel">
		<h2>{$options.mode === 'translate' ? 'Translation engine' : 'Transcription engine'}</h2>
		{#if $options.mode === 'translate'}
			<div class="segmented sub">
				<button disabled={controlsLocked} class:active={$options.provider === 'gemini'} onclick={() => setProvider('gemini')}>
					Google Gemini
				</button>
				<button disabled={controlsLocked} class:active={$options.provider === 'openai'} onclick={() => setProvider('openai')}>
					OpenAI
				</button>
			</div>

			{#if $options.provider === 'gemini'}
				<p class="hint engine-hint">
					<code>gemini-3.5-live-translate-preview</code> — translated captions come from the
					output transcription; generated audio is discarded.
				</p>
			{:else}
				<p class="hint engine-hint">
					<code>gpt-realtime-translate</code> — OpenAI's dedicated live speech-translation model
					(70+ languages in, 13 out). Captions come from its transcript; audio is discarded. Fixed
					target language.
				</p>
			{/if}
		{:else}
			<p class="hint engine-hint">
				<code>voxtral-mini-transcribe-realtime-2602</code> — 16 kHz realtime speech-to-text
				with a 480 ms target delay. Its text appears directly as subtitles and in the saved transcript.
			</p>
		{/if}
	</section>

	<section class="panel controls">
		{#if $isRunning}
			<button class="danger big" disabled={sessionBusy} onclick={stop}>{sessionBusy ? 'Stopping…' : '■ Stop'}</button>
		{:else}
			<button class="primary big" disabled={!$hasKey || browserMode || sessionBusy} onclick={start}>
				{sessionBusy ? 'Starting…' : $options.mode === 'translate' ? '▶ Start translating' : '▶ Start subtitles'}
			</button>
		{/if}
		{#if $statusMessage}
			<span class="status-msg">{$statusMessage}</span>
		{/if}
		<div class="spacer"></div>
		<div class="font-ctl" title="Caption font size on the overlay">
			<span class="font-label">Caption size</span>
			<button class="ghost step" onclick={() => setFont($overlayFontSize - 2)}>−</button>
			<span class="font-val">{$overlayFontSize}</span>
			<button class="ghost step" onclick={() => setFont($overlayFontSize + 2)}>+</button>
		</div>
		<button class="ghost" class:active-toggle={moveOverlay} onclick={toggleMoveOverlay}>
			{moveOverlay ? '✓ Done moving' : 'Move overlay'}
		</button>
		<button class="ghost" onclick={() => api.showOverlay(true)}>Show overlay</button>
	</section>

	<TranscriptMonitor
		mode={$options.mode}
		latestCaption={$latestCaption}
		transcript={$transcript}
		onError={(message) => statusMessage.set(message)}
	/>
</main>

<style>
	main {
		max-width: 860px;
		margin: 0 auto;
		padding: 20px;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	h1 {
		margin: 0;
		font-size: 22px;
	}
	.subtitle {
		color: var(--muted);
		font-size: 13px;
	}
	h2 {
		margin: 0 0 10px;
		font-size: 14px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--muted);
	}
	.panel {
		background: var(--panel);
		border: 1px solid var(--border);
		border-radius: 12px;
		padding: 16px;
	}
	.grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 16px;
	}
	.banner,
	.hint {
		color: var(--muted);
		font-size: 13px;
	}
	.banner {
		background: var(--panel-2);
		border: 1px solid var(--border);
		border-radius: 10px;
		padding: 12px;
	}
	code {
		background: var(--panel-2);
		padding: 1px 5px;
		border-radius: 4px;
		font-size: 0.9em;
	}
	select {
		background: var(--panel-2);
		border: 1px solid var(--border);
		color: var(--text);
		border-radius: 8px;
		padding: 9px 11px;
		width: 100%;
	}
	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin-top: 12px;
		font-size: 13px;
		color: var(--muted);
	}
	.segmented {
		display: flex;
		gap: 6px;
		background: var(--panel-2);
		padding: 4px;
		border-radius: 10px;
		border: 1px solid var(--border);
	}
	.segmented button {
		flex: 1;
		background: transparent;
		border: 0;
		color: var(--muted);
		padding: 9px;
		border-radius: 7px;
	}
	.segmented button.active {
		background: var(--accent);
		color: white;
	}
	.flip {
		margin-top: 12px;
	}
	.engine-hint {
		margin: 10px 0 0;
	}
	.segmented.sub {
		margin-bottom: 10px;
	}
	.meters {
		margin-top: 14px;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	button.primary {
		background: var(--accent);
		color: white;
		border: 0;
		border-radius: 8px;
		padding: 9px 16px;
	}
	button.danger {
		background: var(--danger);
		color: white;
		border: 0;
		border-radius: 8px;
		padding: 9px 16px;
	}
	button.ghost {
		background: transparent;
		color: var(--text);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 9px 14px;
	}
	button.ghost.active-toggle {
		border-color: var(--accent);
		color: var(--accent);
	}
	.big {
		font-size: 16px;
		padding: 12px 22px;
	}
	.controls {
		display: flex;
		align-items: center;
		gap: 14px;
	}
	.spacer {
		flex: 1;
	}
	.status-msg {
		color: var(--warn);
		font-size: 13px;
	}
	.font-ctl {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.font-label {
		font-size: 12px;
		color: var(--muted);
	}
	.font-val {
		min-width: 28px;
		text-align: center;
		font-variant-numeric: tabular-nums;
	}
	.step {
		padding: 6px 12px;
		line-height: 1;
	}
	.state {
		display: flex;
		align-items: center;
		gap: 7px;
		font-size: 13px;
		color: var(--muted);
	}
	.state .dot {
		width: 9px;
		height: 9px;
		border-radius: 50%;
		background: var(--muted);
	}
	.state-running .dot {
		background: var(--accent-2);
		box-shadow: 0 0 8px var(--accent-2);
	}
	.state-connecting .dot,
	.state-reconnecting .dot {
		background: var(--warn);
	}
	.state-error .dot {
		background: var(--danger);
	}
	@media (max-width: 680px) {
		.grid {
			grid-template-columns: 1fr;
		}
	}
</style>
