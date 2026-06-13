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
		pushCaption
	} from '$lib/stores';
	import type { AudioDevice, AudioSource, TargetLanguage } from '$lib/types';
	import LevelMeter from '$lib/LevelMeter.svelte';

	let microphones = $state<AudioDevice[]>([]);
	let apiKeyInput = $state('');
	let savingKey = $state(false);
	let browserMode = $state(false);

	const running = $derived($sessionState === 'running' || $sessionState === 'reconnecting');

	onMount(() => {
		browserMode = !isTauri();
		if (browserMode) return;

		void refresh();

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
			[microphones, $hasKey] = await Promise.all([api.listMicrophones(), api.hasApiKey()]);
		} catch (e) {
			statusMessage.set(String(e));
		}
	}

	async function saveKey() {
		if (!apiKeyInput.trim()) return;
		savingKey = true;
		try {
			await api.setApiKey(apiKeyInput.trim());
			apiKeyInput = '';
			$hasKey = true;
		} catch (e) {
			statusMessage.set(String(e));
		} finally {
			savingKey = false;
		}
	}

	async function clearKey() {
		await api.clearApiKey();
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

	// Quick flip of the caption language — handy when speakers alternate.
	function flipDirection() {
		setTarget($options.targetLanguage === 'en' ? 'fr' : 'en');
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

	{#if !$hasKey && !browserMode}
		<section class="panel key">
			<h2>Gemini API key</h2>
			<p class="hint">
				Stored in the OS keychain, used only from the Rust core. Needs access to
				<code>gemini-3.5-live-translate-preview</code>.
			</p>
			<div class="row">
				<input
					type="password"
					placeholder="Paste your Gemini API key"
					bind:value={apiKeyInput}
					onkeydown={(e) => e.key === 'Enter' && saveKey()}
				/>
				<button class="primary" disabled={savingKey || !apiKeyInput.trim()} onclick={saveKey}>
					{savingKey ? 'Saving…' : 'Save key'}
				</button>
			</div>
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
			<p class="hint">Spoken language is auto-detected; pick the language the audience reads.</p>
			<div class="segmented">
				<button class:active={$options.targetLanguage === 'en'} onclick={() => setTarget('en')}>
					🇬🇧 English
				</button>
				<button class:active={$options.targetLanguage === 'fr'} onclick={() => setTarget('fr')}>
					🇫🇷 Français
				</button>
			</div>
			<button class="ghost flip" onclick={flipDirection}>⇄ Flip (F2)</button>
		</section>
	</div>

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
		<button class="ghost" onclick={() => api.showOverlay(true)}>Show overlay</button>
	</section>

	<section class="panel monitor">
		<h2>Live monitor</h2>
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
				{#each $transcript as line (line.turnId + line.origin)}
					<li>{line.text}</li>
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
