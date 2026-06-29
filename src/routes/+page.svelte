<script lang="ts">
	import { onMount } from 'svelte';
	import { api, on, isTauri } from '$lib/tauri';
	import {
		sessionState,
		statusMessage,
		hasKey,
		options,
		latestCaption,
		transcript,
		micLevel,
		systemLevel,
		overlayFontSize,
		pushCaption
	} from '$lib/stores';
	import type {
		AudioDevice,
		AudioSource,
		Provider,
		TargetLanguage,
		TranslationMode
	} from '$lib/types';
	import LevelMeter from '$lib/LevelMeter.svelte';

	let microphones = $state<AudioDevice[]>([]);
	let apiKeyInput = $state('');
	let savingKey = $state(false);
	let editingKey = $state(false);
	let browserMode = $state(false);

	const running = $derived($sessionState === 'running' || $sessionState === 'reconnecting');

	onMount(() => {
		browserMode = !isTauri();
		if (browserMode) return;

		void refresh();
		// Sync the overlay to the operator's current caption size on load.
		void api.setOverlayConfig({ fontSize: $overlayFontSize });

		const unlisteners: Array<Promise<() => void>> = [
			on.caption((c) => pushCaption(c)),
			on.level((l) => (l.source === 'microphone' ? micLevel.set(l) : systemLevel.set(l))),
			on.status((s) => {
				sessionState.set(s.state);
				statusMessage.set(s.message ?? '');
			})
		];

		return () => {
			void Promise.all(unlisteners).then((fns) => fns.forEach((f) => f()));
		};
	});

	async function refresh() {
		try {
			microphones = await api.listMicrophones();
			await checkKey();
		} catch (e) {
			statusMessage.set(String(e));
		}
	}

	async function checkKey() {
		try {
			$hasKey = await api.hasApiKey($options.provider);
		} catch (e) {
			statusMessage.set(String(e));
		}
	}

	async function saveKey() {
		if (!apiKeyInput.trim()) return;
		savingKey = true;
		try {
			await api.setApiKey($options.provider, apiKeyInput.trim());
			apiKeyInput = '';
			$hasKey = true;
			editingKey = false;
		} catch (e) {
			statusMessage.set(String(e));
		} finally {
			savingKey = false;
		}
	}

	async function clearKey() {
		await api.clearApiKey($options.provider);
		$hasKey = false;
	}

	async function start() {
		try {
			await api.startSession($options);
		} catch (e) {
			statusMessage.set(String(e));
		}
	}

	async function stop() {
		try {
			await api.stopSession();
		} catch (e) {
			statusMessage.set(String(e));
		}
	}

	function setSource(s: AudioSource) {
		$options = { ...$options, source: s };
	}

	function setTarget(t: TargetLanguage) {
		$options = { ...$options, targetLanguage: t };
	}

	function setMode(m: TranslationMode) {
		$options = { ...$options, mode: m };
	}

	function setAuto(on: boolean) {
		$options = { ...$options, autoBidirectional: on };
	}

	function setProvider(p: Provider) {
		if (p === $options.provider) return;
		$options = { ...$options, provider: p };
		// The key panel and Start gating follow the active provider's key.
		editingKey = false;
		apiKeyInput = '';
		void checkKey();
	}

	const engineHint = $derived(
		$options.mode === 'live-translate'
			? 'Dedicated speech translation model. Captions come from its transcript; audio is discarded.'
			: 'General model: audio in → translated text out. No audio generated; more promptable.'
	);

	// Provider-specific key panel copy.
	const keyInfo = $derived(
		$options.provider === 'openai'
			? {
					name: 'OpenAI',
					model: 'gpt-realtime-translate',
					url: 'https://platform.openai.com/api-keys'
				}
			: {
					name: 'Gemini',
					model: 'gemini-3.5-live-translate-preview',
					url: 'https://aistudio.google.com/apikey'
				}
	);

	// Quick flip of the caption language — handy when speakers alternate.
	function flipDirection() {
		setTarget($options.targetLanguage === 'en' ? 'fr' : 'en');
	}

	// Caption size: update the store (persists) and push it live to the overlay.
	function setFont(size: number) {
		const clamped = Math.max(20, Math.min(96, Math.round(size)));
		overlayFontSize.set(clamped);
		void api.setOverlayConfig({ fontSize: clamped });
	}

	let savedPath = $state('');

	const pad = (n: number) => String(n).padStart(2, '0');

	async function saveTranscript() {
		const lines = [...$transcript].reverse(); // chronological order
		if (!lines.length) return;
		const now = new Date();
		const header = `# Live translation transcript\n\n${now.toLocaleString()} · STIAS DH & AI workshop\n\n`;
		const body = lines
			.map((l) => {
				const src = l.sourceText ? `\n  - _source_: ${l.sourceText}` : '';
				return `- **${l.time}** · ${l.origin}\n  - ${l.text}${src}`;
			})
			.join('\n');
		const stamp = `${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}-${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}`;
		try {
			savedPath = await api.saveTranscript(`${header}${body}\n`, `transcript-${stamp}.md`);
		} catch (e) {
			statusMessage.set(String(e));
		}
	}

	function clearTranscript() {
		transcript.set([]);
		savedPath = '';
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
			<h1>Live Translation</h1>
			<span class="subtitle">FR ⇄ EN · STIAS DH &amp; AI workshop</span>
		</div>
		<div class="state state-{$sessionState}">
			<span class="dot"></span>
			{stateLabel[$sessionState] ?? $sessionState}
		</div>
	</header>

	{#if browserMode}
		<div class="banner">
			Running in a browser without the Tauri runtime — controls are disabled. Launch with
			<code>npm run tauri dev</code> for audio capture and translation.
		</div>
	{/if}

	{#if !browserMode}
		<section class="panel key">
			<h2>{keyInfo.name} API key</h2>
			{#if $hasKey && !editingKey}
				<div class="row key-saved">
					<span class="key-ok">✓ Saved to the OS keychain</span>
					<div class="spacer"></div>
					<button class="ghost" onclick={() => { editingKey = true; apiKeyInput = ''; }}>
						Replace
					</button>
					<button class="ghost remove" onclick={clearKey}>Remove</button>
				</div>
			{:else}
				<p class="hint">
					Stored in the OS keychain, used only from the Rust core. Needs access to
					<code>{keyInfo.model}</code>. Get one at
						<a href={keyInfo.url} target="_blank" rel="noreferrer">{keyInfo.url}</a>.
				</p>
				<div class="row">
					<input
						type="password"
						placeholder="Paste your {keyInfo.name} API key"
						bind:value={apiKeyInput}
						onkeydown={(e) => e.key === 'Enter' && saveKey()}
					/>
					<button class="primary" disabled={savingKey || !apiKeyInput.trim()} onclick={saveKey}>
						{savingKey ? 'Saving…' : 'Save key'}
					</button>
					{#if $hasKey}
						<button class="ghost" onclick={() => { editingKey = false; apiKeyInput = ''; }}>
							Cancel
						</button>
					{/if}
				</div>
			{/if}
		</section>
	{/if}

	<div class="grid">
		<section class="panel">
			<h2>Audio source</h2>
			<div class="segmented">
				<button class:active={$options.source === 'microphone'} onclick={() => setSource('microphone')}>
					🎤 Microphone
				</button>
				<button class:active={$options.source === 'system'} onclick={() => setSource('system')}>
					🔊 System (Zoom)
				</button>
				<button class:active={$options.source === 'both'} onclick={() => setSource('both')}>
					Both
				</button>
			</div>

			{#if $options.source !== 'system'}
				<label class="field">
					<span>Microphone device</span>
					<select
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
			<h2>Caption language</h2>
			<p class="hint">
				{$options.autoBidirectional
					? 'French ↔ English, chosen automatically per speaker. Pick the fallback for any other language:'
					: 'Spoken language is auto-detected; pick the language the audience reads.'}
			</p>
			<div class="segmented">
				<button class:active={$options.targetLanguage === 'en'} onclick={() => setTarget('en')}>
					🇬🇧 English
				</button>
				<button class:active={$options.targetLanguage === 'fr'} onclick={() => setTarget('fr')}>
					🇫🇷 Français
				</button>
			</div>
			<button class="ghost flip" onclick={flipDirection}>⇄ Flip (F2)</button>

			<label class="auto-toggle">
				<input
					type="checkbox"
					checked={$options.autoBidirectional}
					onchange={(e) => setAuto(e.currentTarget.checked)}
				/>
				<span>🔁 Auto (FR ⇄ EN) — caption each speaker in the <em>other</em> language</span>
			</label>
			{#if $options.autoBidirectional && !($options.provider === 'gemini' && $options.mode === 'speech-to-text')}
				<p class="hint warn-hint">
					Auto direction only applies to Gemini's Speech → Text engine; the selected engine will
					use the fixed language above.
				</p>
			{/if}
		</section>
	</div>

	<section class="panel">
		<h2>Translation engine</h2>
			<div class="segmented sub">
				<button class:active={$options.provider === 'gemini'} onclick={() => setProvider('gemini')}>
					Google Gemini
				</button>
				<button class:active={$options.provider === 'openai'} onclick={() => setProvider('openai')}>
					OpenAI
				</button>
			</div>

			{#if $options.provider === 'gemini'}
				<div class="segmented">
					<button
						class:active={$options.mode === 'live-translate'}
						onclick={() => setMode('live-translate')}
					>
						Live Translate (speech model)
					</button>
					<button
						class:active={$options.mode === 'speech-to-text'}
						onclick={() => setMode('speech-to-text')}
					>
						Speech → Text (general)
					</button>
				</div>
				<p class="hint engine-hint">{engineHint}</p>
			{:else}
				<p class="hint engine-hint">
					<code>gpt-realtime-translate</code> — OpenAI's dedicated live speech-translation model
					(70+ languages in, 13 out). Captions come from its transcript; audio is discarded. Fixed
					target language.
				</p>
			{/if}
	</section>

	<section class="panel controls">
		{#if running}
			<button class="danger big" onclick={stop}>■ Stop</button>
		{:else}
			<button
				class="primary big"
				disabled={!$hasKey || browserMode || $sessionState === 'connecting'}
				onclick={start}
			>
				▶ Start translating
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
		<button class="ghost" onclick={() => api.showOverlay(true)}>Show overlay</button>
	</section>

	<section class="panel monitor">
		<div class="monitor-head">
			<h2>Live monitor</h2>
			<div class="monitor-actions">
				<button class="ghost" disabled={!$transcript.length} onclick={saveTranscript}>
					Save transcript
				</button>
				<button class="ghost" disabled={!$transcript.length} onclick={clearTranscript}>
					Clear
				</button>
			</div>
		</div>
		{#if savedPath}
			<p class="saved">Saved to <code>{savedPath}</code></p>
		{/if}
		{#if $latestCaption}
			<div class="current">
				<div class="src">{$latestCaption.sourceText}</div>
				<div class="trans" class:interim={!$latestCaption.final}>{$latestCaption.text}</div>
			</div>
		{:else}
			<p class="hint">Translated captions will appear here and on the overlay.</p>
		{/if}

		{#if $transcript.length}
			<ul class="log">
				{#each $transcript as line (line.time + line.text)}
					<li><span class="log-time">{line.time}</span> {line.text}</li>
				{/each}
			</ul>
		{/if}
	</section>
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
	.row {
		display: flex;
		gap: 8px;
	}
	.key-saved {
		align-items: center;
	}
	.key-ok {
		color: var(--accent-2);
		font-size: 14px;
	}
	button.ghost.remove {
		color: var(--danger);
		border-color: var(--danger);
	}
	input,
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
	.auto-toggle {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-top: 14px;
		font-size: 13px;
		color: var(--text);
		cursor: pointer;
	}
	.auto-toggle input {
		width: auto;
		cursor: pointer;
	}
	.auto-toggle em {
		font-style: italic;
	}
	.warn-hint {
		color: var(--warn);
		margin-top: 8px;
	}
	.engine-hint {
		margin: 10px 0 0;
	}
	.segmented.sub {
		margin-bottom: 10px;
	}
	.hint a {
		color: var(--accent-2);
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
	.monitor-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.monitor-actions {
		display: flex;
		gap: 8px;
	}
	.saved {
		font-size: 12px;
		color: var(--accent-2);
		margin: 0 0 10px;
		word-break: break-all;
	}
	.log-time {
		color: var(--muted);
		font-variant-numeric: tabular-nums;
		margin-right: 6px;
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
	.current {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.current .src {
		color: var(--muted);
		font-size: 14px;
	}
	.current .trans {
		font-size: 22px;
		line-height: 1.3;
	}
	.current .trans.interim {
		opacity: 0.65;
		font-style: italic;
	}
	.log {
		list-style: none;
		margin: 14px 0 0;
		padding: 0;
		max-height: 180px;
		overflow-y: auto;
		border-top: 1px solid var(--border);
	}
	.log li {
		padding: 7px 0;
		border-bottom: 1px solid var(--border);
		font-size: 14px;
		color: var(--muted);
	}
	@media (max-width: 680px) {
		.grid {
			grid-template-columns: 1fr;
		}
	}
</style>
