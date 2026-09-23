<script lang="ts">
	import { api, isTauri } from './tauri';
	import { t } from './i18n';
	import { asStatus, type AppError } from './errors';
	import { PROVIDER_META } from './providers';
	import { providerKeyName } from './types';
	import ChecklistRow from './ui/ChecklistRow.svelte';
	import type { Provider } from './types';

	interface Props {
		provider: Provider;
		locked?: boolean;
		onAvailability: (provider: Provider, available: boolean) => void;
		onError: (message: string | AppError) => void;
	}

	let { provider, locked = false, onAvailability, onError }: Props = $props();
	// The key field's label is visually carried by the row title beside it; these tie the two
	// together for a screen reader, which can land on the input without ever seeing the row.
	// One `$props.id()` per component, suffixed — Svelte allows exactly one call.
	const uid = $props.id();
	const fieldId = `${uid}-key`;
	const descId = `${uid}-desc`;
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
			onError(asStatus(error));
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
			onError(asStatus(error));
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
			onError(asStatus(error));
		}
	}
</script>

{#if available && !editing}
	<ChecklistRow status="ok" title={$t.key.title(keyName)} desc={$t.key.saved}>
		{#snippet action()}
			<div class="actions">
				<button
					class="ghost"
					disabled={locked}
					onclick={() => {
						editing = true;
						apiKeyInput = '';
					}}
				>
					{$t.key.replace}
				</button>
				<button class="ghost" disabled={locked} onclick={clearKey}>{$t.key.remove}</button>
			</div>
		{/snippet}
	</ChecklistRow>
{:else}
	<ChecklistRow status="wait" pending title={$t.key.title(keyName)} titleFor={fieldId} {descId}>
		{#snippet description()}
			{$t.key.desc.before}
			<code>{meta.modelId}</code>{$t.key.desc.after}
			{#if meta.keyUrl}
				<a href={meta.keyUrl} target="_blank" rel="noopener noreferrer">
					{$t.key.getKey}<span class="sr-only">{$t.key.opensInBrowser}</span>
				</a>
			{/if}
		{/snippet}
		{#snippet action()}
			<div class="actions">
				<input
					id={fieldId}
					type="password"
					placeholder={$t.key.placeholder(keyName)}
					aria-describedby={descId}
					autocomplete="off"
					autocapitalize="off"
					autocorrect="off"
					spellcheck="false"
					bind:value={apiKeyInput}
					disabled={locked}
					onkeydown={(event) => event.key === 'Enter' && void saveKey()}
				/>
				<button
					class="save"
					disabled={locked || saving || !apiKeyInput.trim()}
					aria-busy={saving}
					onclick={saveKey}
				>
					{saving ? $t.key.saving : $t.key.save}
				</button>
				{#if available}
					<button
						class="ghost"
						onclick={() => {
							editing = false;
							apiKeyInput = '';
						}}
					>
						{$t.key.cancel}
					</button>
				{/if}
			</div>
		{/snippet}
	</ChecklistRow>
{/if}

<style>
	code {
		font-family: var(--font-mono);
		font-size: var(--type-caption);
		color: var(--text-secondary);
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
		font-size: var(--type-small);
		font-weight: 500;
		line-height: 1;
		padding: 7px 11px;
		border-radius: var(--radius-control);
	}
	button.ghost {
		color: var(--text-secondary);
		border: 1px solid var(--line-strong);
		background: transparent;
	}
	button.ghost:hover:not(:disabled) {
		border-color: var(--line-hover);
		color: var(--text-body);
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
		/* 220px at 100%. A key is a long opaque string, so the field is sized in the text
		   it holds rather than in pixels that stop matching it. */
		width: 13.75em;
		max-width: 100%;
		background: var(--surface-1);
		border: 1px solid var(--line-strong);
		color: var(--text-body);
		border-radius: var(--radius-control);
		padding: 7px 10px;
		font-size: var(--type-small);
	}
	/* The mint border says "this field is where you are typing"; the ring from app.css stays,
	   because on a dark panel a border tint alone is not a focus indicator. */
	input:focus {
		border-color: var(--accent-border);
	}
</style>
