<script lang="ts">
	import { api, isTauri } from './tauri';
	import { PROVIDER_META } from './providers';
	import { providerKeyName } from './types';
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

	const meta = $derived(PROVIDER_META[provider]);

	// The row title wants the credential's short name, not the full product name. Both Gemini
	// backends read the same stored key, so switching mode never re-prompts for it.
	const keyName = $derived(providerKeyName(provider));

	async function checkKey(activeProvider: Provider) {
		// Second line of defence behind the caller's `browserMode` check: a browser preview has
		// no Credential Manager to ask, and the effect below has already reported "no key", so
		// there is nothing to do but leave it at that. Invoking anyway would surface a Tauri IPC
		// error the operator can neither act on nor dismiss.
		if (!isTauri()) return;
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

<div class="row" class:pending={!available || editing}>
	{#if available && !editing}
		<span class="mark ok">
			<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" aria-hidden="true"><path d="M4 12.5l5 5L20 6.5" /></svg>
		</span>
		<div class="body">
			<span class="title">{keyName} key</span>
			<span class="desc">Saved in Windows Credential Manager · read only by the Rust core</span>
		</div>
		<div class="actions">
			<button class="ghost" disabled={locked} onclick={() => { editing = true; apiKeyInput = ''; }}>Replace</button>
			<button class="ghost" disabled={locked} onclick={clearKey}>Remove</button>
		</div>
	{:else}
		<span class="mark wait"><span class="dot"></span></span>
		<div class="body">
			<span class="title">{keyName} key</span>
			<span class="desc">
				Stored in Windows Credential Manager, used only from the Rust core. Needs access to
				<code>{meta.modelId}</code>.
				{#if meta.keyUrl}
					<a href={meta.keyUrl} target="_blank" rel="noreferrer">Get a key</a>
				{/if}
			</span>
		</div>
		<div class="actions">
			<input
				type="password"
				placeholder="Paste your {keyName} API key"
				bind:value={apiKeyInput}
				disabled={locked}
				onkeydown={(event) => event.key === 'Enter' && void saveKey()}
			/>
			<button class="save" disabled={locked || saving || !apiKeyInput.trim()} onclick={saveKey}>
				{saving ? 'Saving…' : 'Save'}
			</button>
			{#if available}
				<button class="ghost" onclick={() => { editing = false; apiKeyInput = ''; }}>Cancel</button>
			{/if}
		</div>
	{/if}
</div>

<style>
	.row {
		display: grid;
		grid-template-columns: 24px 1fr auto;
		align-items: center;
		gap: 14px;
		padding: 15px 0;
		border-bottom: 1px solid var(--hairline);
	}
	/* While a key is being entered the description wraps to two lines, so the row's parts
	   align to the top rather than to a shifting centre. */
	.row.pending {
		align-items: start;
	}
	.mark {
		width: 20px;
		height: 20px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.row.pending .mark {
		margin-top: 3px;
	}
	.mark.ok {
		background: var(--accent-chip-bg);
		color: var(--accent);
	}
	.mark.wait {
		background: var(--warn-bg);
		color: var(--warn);
	}
	.mark .dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: currentColor;
	}
	.body {
		display: flex;
		flex-direction: column;
		gap: 3px;
		min-width: 0;
	}
	.title {
		font-size: 13.5px;
		font-weight: 500;
		line-height: 1.2;
	}
	.desc {
		font-size: 12px;
		line-height: 1.35;
		color: var(--muted-2);
	}
	code {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-dim);
	}
	a {
		color: var(--accent-soft);
		text-decoration: none;
	}
	a:hover {
		text-decoration: underline;
	}
	.actions {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	button {
		font-size: 11.5px;
		font-weight: 500;
		line-height: 1;
		padding: 7px 11px;
		border-radius: 7px;
	}
	button.ghost {
		color: var(--text-soft);
		border: 1px solid var(--border);
		background: transparent;
	}
	button.ghost:hover:not(:disabled) {
		border-color: var(--border-hover);
		color: var(--text);
	}
	button.save {
		border: 0;
		background: linear-gradient(#5ad1a0, #43b989);
		color: var(--on-accent);
		font-weight: 600;
		padding: 8px 13px;
	}
	button.save:hover:not(:disabled) {
		filter: brightness(1.06);
	}
	input {
		width: 220px;
		background: var(--panel-2);
		border: 1px solid var(--border);
		color: var(--text);
		border-radius: 7px;
		padding: 7px 10px;
		font-size: 12.5px;
	}
	input:focus {
		outline: none;
		border-color: var(--accent-border);
	}
	/* Matches the tightened checklist rhythm the stage adopts on a short window. */
	@media (max-height: 740px) {
		.row {
			padding: 12px 0;
		}
	}
</style>
