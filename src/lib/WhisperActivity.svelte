<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from './i18n';
	import { isRunning, statusMessage } from './stores';
	import { api, on } from './tauri';
	import { asStatus } from './errors';
	import type { WhisperProgress, Origin } from './types';
	import ToolButton from './ui/ToolButton.svelte';

	let progress = $state<Partial<Record<Origin, WhisperProgress>>>({});
	let confirmDiscard = $state(false);
	const pending = $derived(
		Math.ceil(Math.max(0, ...Object.values(progress).map((p) => p.pendingMs)) / 1000)
	);
	const finalizing = $derived(Object.values(progress).some((p) => p.finalizing));
	$effect(() => {
		if (!$isRunning) progress = {};
	});
	onMount(() => {
		let disposed = false;
		let cleanup: (() => void) | undefined;
		void on
			.whisperProgress((p) => (progress = { ...progress, [p.origin]: p }))
			.then((fn) => {
				if (disposed) fn();
				else cleanup = fn;
			});
		return () => {
			disposed = true;
			cleanup?.();
		};
	});
	async function discard() {
		confirmDiscard = false;
		try {
			await api.discardWhisperPending();
		} catch (error) {
			statusMessage.set(asStatus(error));
		}
	}
</script>

{#if $isRunning && (pending > 0 || finalizing)}
	<div class="local-progress" role="status">
		<span>{finalizing ? $t.whisper.processing : $t.whisper.pending} · {pending} s</span>
		{#if finalizing && pending > 0}
			<ToolButton onclick={() => (confirmDiscard = true)}>{$t.whisper.discard}</ToolButton>
		{/if}
	</div>
{/if}
{#if confirmDiscard}
	<div class="discard-confirm" role="alert">
		<p>{$t.whisper.discardConfirm}</p>
		<ToolButton onclick={discard}>{$t.whisper.discard}</ToolButton>
		<ToolButton onclick={() => (confirmDiscard = false)}>{$t.history.cancel}</ToolButton>
	</div>
{/if}

<style>
	.local-progress,
	.discard-confirm {
		padding: var(--space-3);
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
		font-size: var(--type-small);
	}
	.local-progress {
		display: flex;
		gap: var(--space-3);
		align-items: center;
		justify-content: space-between;
	}
</style>
