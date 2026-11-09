<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from './i18n';
	import { options, whisperModels, statusMessage } from './stores';
	import { api } from './tauri';
	import { whisperEngine as engine } from './whisperProgress';
	import { asStatus } from './errors';
	import { WHISPER_MODELS, whisperModelTranslates, type WhisperModelId } from './types';
	import Field from './ui/Field.svelte';
	import Preference from './ui/Preference.svelte';
	import Select from './ui/Select.svelte';
	import ToolButton from './ui/ToolButton.svelte';

	let { locked, browserMode }: { locked: boolean; browserMode: boolean } = $props();
	let busy = $state(false);
	let refreshing = false;
	let disposed = false;
	const selected = $derived($options.whisperModel ?? 'base');
	const model = $derived($whisperModels.find((m) => m.id === selected));
	const downloading = $derived($whisperModels.some((m) => m.downloading));
	const percent = $derived(model ? Math.floor((model.downloadedBytes / model.bytes) * 100) : 0);
	const translating = $derived($options.provider === 'whisper-translate');
	// Turbo only transcribes, so translation does not offer it; a Turbo chosen for subtitles
	// stays chosen, and the note below says why translation cannot start with it.
	const offered = $derived(
		WHISPER_MODELS.filter((id) => !translating || whisperModelTranslates(id) || id === selected)
	);

	async function refresh() {
		if (browserMode || disposed || refreshing) return;
		refreshing = true;
		try {
			const models = await api.whisperModels();
			if (!disposed) whisperModels.set(models);
		} catch (error) {
			if (!disposed) statusMessage.set(asStatus(error));
		} finally {
			refreshing = false;
		}
	}
	async function download() {
		if (locked || busy || downloading || browserMode) return;
		busy = true;
		statusMessage.set('');
		try {
			await api.downloadWhisperModel(selected);
		} catch (error) {
			statusMessage.set(asStatus(error));
		} finally {
			busy = false;
			await refresh();
		}
	}
	async function remove() {
		if (locked || busy || downloading || browserMode) return;
		busy = true;
		try {
			await api.removeWhisperModel(selected);
		} catch (error) {
			statusMessage.set(asStatus(error));
		} finally {
			busy = false;
			await refresh();
		}
	}
	async function cancel() {
		try {
			await api.cancelWhisperDownload();
		} catch (error) {
			statusMessage.set(asStatus(error));
		}
	}
	onMount(() => {
		void refresh();
		const timer = setInterval(() => {
			if (busy || downloading) void refresh();
		}, 500);
		return () => {
			disposed = true;
			clearInterval(timer);
		};
	});
</script>

<div class="models">
	<Field label={$t.whisper.model}>
		<Select
			value={selected}
			disabled={locked || busy || downloading}
			onchange={(e) =>
				($options = { ...$options, whisperModel: e.currentTarget.value as WhisperModelId })}
		>
			{#each offered as id (id)}
				<option value={id}
					>{$t.whisper[id]}{#if $whisperModels.find((m) => m.id === id)}
						— {Math.ceil($whisperModels.find((m) => m.id === id)!.bytes / 1048576)} MiB{/if}</option
				>
			{/each}
		</Select>
	</Field>
	<!-- Translation is where the small models fall short, so it is the one place to say so. -->
	<p class="hint">
		{translating
			? whisperModelTranslates(selected)
				? $t.whisper.translateHint
				: $t.whisper.turboCannotTranslate
			: selected === 'turbo'
				? $t.whisper.turboHint
				: $t.whisper.hint}
	</p>
	<!-- A live region, so it says the model's state and nothing that ticks: the download is
	     polled every 500 ms, and the percentage is the progress bar's to carry. -->
	<p class="hint" role="status">
		{model?.downloading
			? $t.whisper.downloading
			: model?.installed
				? $t.whisper.ready
				: $t.whisper.missing}
	</p>
	{#if model?.downloading}
		<progress
			value={model.downloadedBytes}
			max={model.bytes}
			aria-label={$t.whisper.downloading}
			aria-valuetext="{percent}%"
		></progress>
	{/if}
	<div class="actions">
		{#if busy || downloading}
			<ToolButton disabled={locked || !downloading} onclick={cancel}>{$t.whisper.cancel}</ToolButton
			>
		{:else if model?.installed}
			<ToolButton disabled={locked || browserMode || model.inUse} onclick={remove}
				>{$t.whisper.remove}</ToolButton
			>
		{:else}
			<ToolButton disabled={locked || browserMode} onclick={download}
				>{$t.whisper.download}</ToolButton
			>
		{/if}
	</div>
	<Preference
		label={$t.whisper.graphics}
		note={$t.whisper.graphicsNote}
		checked={$options.whisperGpu !== false}
		disabled={locked}
		onchange={(on) => ($options = { ...$options, whisperGpu: on })}
	/>
	{#if $engine}
		<p class="engine">
			{$engine.backend === 'cpu'
				? $t.whisper.engineProcessor
				: $t.whisper.engineGraphics($engine.device)}
		</p>
		{#if $engine.gpuFailure}
			<p class="engine">
				{$t.whisper.engineFallback} <span class="detail">{$engine.gpuFailure}</span>
			</p>
		{/if}
	{/if}
</div>

<style>
	/* The same 8px rhythm as every other block in the rail: the select, the two notes under
	   it and the button each get their own line, rather than stacking flush. */
	.models {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: var(--space-2);
		margin-top: var(--space-3);
	}
	.models > :global(.ui-field) {
		align-self: stretch;
	}
	progress {
		width: 100%;
		accent-color: var(--accent);
	}
	.actions {
		display: flex;
		gap: var(--space-2);
	}
	.engine {
		margin: 0;
		font-size: var(--type-small);
		color: var(--text-secondary);
	}
	.engine .detail {
		color: var(--text-muted);
		overflow-wrap: anywhere;
	}
</style>
