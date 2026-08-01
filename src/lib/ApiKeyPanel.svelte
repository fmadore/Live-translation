<script lang="ts">
	import { api } from './tauri';
	import type { Provider } from './types';

	interface Props {
		provider: Provider;
		locked?: boolean;
		onAvailability: (provider: Provider, available: boolean) => void;
		onError: (message: string) => void;
	}

	let { provider, locked = false, onAvailability, onError }: Props = $props();
	let apiKeyInput = $state('');
	let saving = $state(false);
	let editing = $state(false);
	let available = $state(false);
	let requestId = 0;

	const keyInfo = $derived(
		provider === 'mistral'
			? { name: 'Mistral', model: 'voxtral-mini-transcribe-realtime-2602', url: 'https://console.mistral.ai/api-keys' }
			: provider === 'openai'
				? { name: 'OpenAI', model: 'gpt-realtime-translate', url: 'https://platform.openai.com/api-keys' }
				: { name: 'Gemini', model: 'gemini-3.5-live-translate-preview', url: 'https://aistudio.google.com/apikey' }
	);

	async function checkKey(activeProvider: Provider) {
		const currentRequest = ++requestId;
		try {
			const result = await api.hasApiKey(activeProvider);
			if (currentRequest !== requestId || provider !== activeProvider) return;
			available = result;
			onAvailability(activeProvider, result);
		} catch (error) {
			onError(String(error));
		}
	}

	$effect(() => {
		const activeProvider = provider;
		editing = false;
		apiKeyInput = '';
		available = false;
		onAvailability(activeProvider, false);
		void checkKey(activeProvider);
	});

	async function saveKey() {
		const key = apiKeyInput.trim();
		if (!key || locked) return;
		saving = true;
		try {
			await api.setApiKey(provider, key);
			apiKeyInput = '';
			available = true;
			editing = false;
			onAvailability(provider, true);
		} catch (error) {
			onError(String(error));
		} finally {
			saving = false;
		}
	}

	async function clearKey() {
		if (locked) return;
		try {
			await api.clearApiKey(provider);
			available = false;
			onAvailability(provider, false);
		} catch (error) {
			onError(String(error));
		}
	}
</script>

<section class="panel key">
	<h2>{keyInfo.name} API key</h2>
	{#if available && !editing}
		<div class="row key-saved">
			<span class="key-ok">✓ Saved to the OS keychain</span>
			<div class="spacer"></div>
			<button class="ghost" disabled={locked} onclick={() => { editing = true; apiKeyInput = ''; }}>Replace</button>
			<button class="ghost remove" disabled={locked} onclick={clearKey}>Remove</button>
		</div>
	{:else}
		<p class="hint">
			Stored in the OS keychain and used only from the Rust core. Needs access to
			<code>{keyInfo.model}</code>. Get a key at
			<a href={keyInfo.url} target="_blank" rel="noreferrer">{keyInfo.url}</a>.
		</p>
		<div class="row">
			<input type="password" placeholder="Paste your {keyInfo.name} API key" bind:value={apiKeyInput}
				disabled={locked} onkeydown={(event) => event.key === 'Enter' && void saveKey()} />
			<button class="primary" disabled={locked || saving || !apiKeyInput.trim()} onclick={saveKey}>
				{saving ? 'Saving…' : 'Save key'}
			</button>
			{#if available}
				<button class="ghost" onclick={() => { editing = false; apiKeyInput = ''; }}>Cancel</button>
			{/if}
		</div>
	{/if}
</section>

<style>
	.panel { background: var(--panel); border: 1px solid var(--border); border-radius: 12px; padding: 16px; }
	h2 { margin: 0 0 10px; font-size: 14px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--muted); }
	.row { display: flex; gap: 8px; }
	.key-saved { align-items: center; }
	.key-ok { color: var(--accent-2); font-size: 14px; }
	.spacer { flex: 1; }
	.hint { color: var(--muted); font-size: 13px; }
	.hint a { color: var(--accent-2); }
	code { background: var(--panel-2); padding: 1px 5px; border-radius: 4px; font-size: 0.9em; }
	input { background: var(--panel-2); border: 1px solid var(--border); color: var(--text); border-radius: 8px; padding: 9px 11px; width: 100%; }
	button.primary { background: var(--accent); color: white; border: 0; border-radius: 8px; padding: 9px 16px; }
	button.ghost { background: transparent; color: var(--text); border: 1px solid var(--border); border-radius: 8px; padding: 9px 14px; }
	button.remove { color: var(--danger); border-color: var(--danger); }
</style>
