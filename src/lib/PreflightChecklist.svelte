<script lang="ts">
	import ApiKeyPanel from './ApiKeyPanel.svelte';
	import ChecklistRow from './ui/ChecklistRow.svelte';
	import { t } from './i18n';
	import { hasKey, options, overlayPlaced, statusMessage } from './stores';
	import { PROVIDER_META, rateText } from './providers';
	import { describeReadiness, providerRequiresKey } from './types';
	import type { OverlayController } from './overlayController.svelte';
	import type { PreflightController } from './preflightController.svelte';

	let {
		preflight,
		overlay,
		browserMode,
		locked,
		usesMic,
		usesSystem
	}: {
		preflight: PreflightController;
		overlay: OverlayController;
		browserMode: boolean;
		/** Setup is locked: a session, a start or a profile load is under way. */
		locked: boolean;
		usesMic: boolean;
		usesSystem: boolean;
	} = $props();

	const needsKey = $derived(providerRequiresKey($options.provider));

	// The demo row's sentence. The core names the readiness state and the catalog words it, so
	// this re-words itself when the interface language changes rather than freezing whatever
	// was true at check time.
	const demoRowText = $derived(describeReadiness(preflight.localReadiness, $t));

	// The tick means "this source has been heard", not "we are listening now". The old check
	// asked the second question from a screen where the answer was always no, because levels
	// only flow once capture is running.
	const audioVerified = $derived(
		$options.provider === 'ondevice' ||
			((!usesMic || preflight.micVerified) && (!usesSystem || preflight.systemVerified))
	);
	// Live view for the duration of a test.
	const audioHearing = $derived(
		(!usesMic || preflight.micSignal) && (!usesSystem || preflight.systemSignal)
	);

	// Which of the four things is under test: the room mic, system loopback, both, or the
	// bundled sample. One key, used for the row title and both tenses of the description.
	const audioSubject = $derived<'microphone' | 'system' | 'both' | 'demo'>(
		$options.provider === 'ondevice'
			? 'demo'
			: $options.source === 'system'
				? 'system'
				: $options.source === 'microphone'
					? 'microphone'
					: 'both'
	);
	const audioCheckDesc = $derived(
		$options.provider === 'ondevice'
			? $t.preflight.audio.heard.demo
			: preflight.audioTesting
				? audioHearing
					? $t.preflight.audio.hearing[audioSubject]
					: $t.preflight.audio.listening
				: audioVerified
					? $t.preflight.audio.heard[audioSubject]
					: $t.preflight.audio.unchecked
	);
</script>

<div class="checklist">
	{#if !needsKey}
		<ChecklistRow
			status={preflight.localReadiness?.ready ? 'ok' : 'wait'}
			title={$t.preflight.demoRow.title}
			desc={demoRowText}
			warn={!preflight.localReadiness?.ready}
		/>
	{:else if !browserMode}
		<ApiKeyPanel
			provider={$options.provider}
			{locked}
			onAvailability={(provider, available) => {
				if ($options.provider === provider) $hasKey = available;
			}}
			onError={(message) => statusMessage.set(message)}
		/>
	{/if}

	<ChecklistRow
		status={audioVerified ? 'ok' : 'wait'}
		title={$t.preflight.audio.title[audioSubject]}
		desc={audioCheckDesc}
		warn={preflight.audioTesting && !audioHearing}
	>
		{#snippet action()}
			<!-- The demo opens no device, so there is nothing to test. Otherwise the row keeps
			     the button in both states: re-checking after moving a cable or switching the
			     room mic is exactly when an operator needs it. -->
			{#if $options.provider === 'ondevice' || browserMode}
				<span></span>
			{:else if preflight.audioTesting}
				<button
					class="adjust"
					disabled={preflight.audioTestBusy}
					aria-busy={preflight.audioTestBusy}
					onclick={preflight.stopAudioTest}
				>
					{$t.preflight.audio.stopTest}
				</button>
			{:else}
				<button
					class="place"
					disabled={preflight.audioTestBusy || locked || !preflight.applicationReady($options)}
					aria-busy={preflight.audioTestBusy}
					onclick={preflight.startAudioTest}
				>
					{audioVerified ? $t.preflight.audio.retest : $t.preflight.audio.test}
				</button>
			{/if}
		{/snippet}
	</ChecklistRow>

	<ChecklistRow
		status={$overlayPlaced ? 'ok' : 'wait'}
		title={$t.preflight.overlay.title}
		desc={$overlayPlaced ? $t.preflight.overlay.placed : $t.preflight.overlay.unplaced}
		warn={!$overlayPlaced}
	>
		{#snippet action()}
			<!-- Placement is never final: re-entering move mode is the way to adjust position and
			     caption size, so the row keeps a button in both states. -->
			<button
				class={$overlayPlaced ? 'adjust' : 'place'}
				aria-pressed={overlay.moveOverlay}
				aria-label={overlay.moveOverlay
					? $t.preflight.overlay.doneLabel
					: $overlayPlaced
						? $t.preflight.overlay.adjustLabel
						: $t.preflight.overlay.placeLabel}
				onclick={overlay.toggleMoveOverlay}
			>
				{overlay.moveOverlay
					? $t.preflight.overlay.done
					: $overlayPlaced
						? $t.preflight.overlay.adjust
						: $t.preflight.overlay.place}
			</button>
		{/snippet}
	</ChecklistRow>

	<ChecklistRow
		status="neutral"
		glyph="$"
		title={$t.preflight.cost.title}
		desc={$options.provider === 'ondevice' ? $t.preflight.cost.free : $t.preflight.cost.billed}
	>
		{#snippet action()}
			<span class="check-rate">{rateText(PROVIDER_META[$options.provider], $t)}</span>
		{/snippet}
	</ChecklistRow>
</div>

<style>
	.checklist {
		flex: 0 0 auto;
		margin-top: 26px;
		border-top: 1px solid var(--hairline);
	}
	.check-rate {
		font-family: var(--font-mono);
		font-size: var(--type-12-5);
		font-weight: 500;
		line-height: 1;
		color: var(--text-soft);
		font-variant-numeric: tabular-nums;
	}
	.place {
		font-size: var(--type-11-5);
		font-weight: 500;
		line-height: 1;
		color: var(--warn-soft);
		padding: 7px 11px;
		border-radius: var(--radius-control);
		border: 1px solid var(--warn-border);
		background: rgba(255, 180, 84, 0.08);
	}
	.place:hover {
		background: var(--warn-bg);
	}
	/* Quiet variant of .place for the already-placed row: same geometry, ghost colours. */
	.adjust {
		font-size: var(--type-11-5);
		font-weight: 500;
		line-height: 1;
		color: var(--text-soft);
		padding: 7px 11px;
		border-radius: var(--radius-control);
		border: 1px solid var(--border);
		background: transparent;
	}
	.adjust:hover {
		border-color: var(--border-hover);
		color: var(--text);
	}
	/* Near the window's minimum height, tighten the vertical rhythm so the checklist and the
	   launch notes still land above the fold. */
	@media (max-height: 740px) {
		.checklist {
			margin-top: 18px;
		}
	}
</style>
