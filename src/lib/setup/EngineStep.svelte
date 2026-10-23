<script lang="ts">
	import WhisperModels from '../WhisperModels.svelte';
	import { t } from '../i18n';
	import { options } from '../stores';
	import { PROVIDER_META, isLocalWhisper, modelLabel, rateParts } from '../providers';
	import { preflightContext } from '../operatorContext';
	import type { Provider } from '../types';
	import type { SetupActions } from '../setupActions';

	/** Step 04: the engines that serve the chosen mode, with what each costs. */
	let {
		actions,
		locked,
		browserMode
	}: { actions: SetupActions; locked: boolean; browserMode: boolean } = $props();
	const preflight = preflightContext();

	// Each mode is served by its own backends; this step lists the ones for the current mode.
	const modeProviders = $derived<Provider[]>(
		$options.mode === 'translate'
			? ['gemini', 'openai', 'whisper-translate']
			: ['whisper', 'mistral', 'gemini-transcribe', 'ondevice']
	);
</script>

<section class="rail-section">
	<div class="step-head">
		<span class="step-no">04</span>
		<h2 class="kicker">{$t.rail.step.engine}</h2>
	</div>
	<div class="engines">
		{#each modeProviders as id (id)}
			{@const p = PROVIDER_META[id]}
			{@const rate = rateParts(p, $t)}
			<!-- Whisper on a processor it was not built for stays listed, so the operator learns
			     why it is missing, but cannot be chosen. -->
			{@const refused = isLocalWhisper(id) && preflight.whisperRefused}
			<button
				class="engine ui-card"
				disabled={locked || refused}
				aria-pressed={$options.provider === id}
				onclick={() => actions.setProvider(id)}
			>
				<span class="engine-body">
					<span class="engine-name">{$t.provider.vendor[id]}</span>
					<span class="engine-model">{modelLabel(p, $t)}</span>{#if id === 'gemini-transcribe'}<span
							class="hint">{$t.design.smartSummary}</span
						>{/if}{#if refused}<span class="hint"
							>{$t.whisper.cpuUnsupported(preflight.whisperCpuMissing)}</span
						>{/if}
				</span>
				<span class="engine-rate">{rate[0]}<span class="unit">{rate[1]}</span></span>
			</button>
		{/each}
	</div>
	{#if isLocalWhisper($options.provider)}<WhisperModels {locked} {browserMode} />{/if}
</section>

<style>
	/* Direct children of the rail's scrolling column keep their height rather than
	   compress, so a short window scrolls instead of clipping text. */
	.kicker {
		flex: 0 0 auto;
	}

	.engines {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.engine {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-3);
	}
	.engine-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		min-width: 0;
	}
	.engine-name {
		font-size: var(--type-body);
		font-weight: 500;
		line-height: 1;
		color: var(--text-secondary);
	}
	.engine[aria-pressed='true'] .engine-name {
		color: var(--text-body);
		font-weight: 600;
	}
	.engine-model {
		font-family: var(--font-mono);
		font-size: var(--type-caption);
		line-height: var(--leading-tight);
		color: var(--text-muted);
		overflow-wrap: anywhere;
	}
	.engine-rate {
		margin-left: auto;
		font-family: var(--font-mono);
		font-size: var(--type-small);
		font-weight: 500;
		line-height: 1;
		color: var(--text-muted);
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}
	.engine[aria-pressed='true'] .engine-rate {
		color: var(--accent-soft);
	}
	/* The unit stays grey when the selected engine turns its rate mint. */
	.engine-rate .unit {
		color: var(--text-muted);
	}
</style>
