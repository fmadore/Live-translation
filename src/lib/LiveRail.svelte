<script lang="ts">
	import CaptionAppearance from './CaptionAppearance.svelte';
	import LevelMeter from './LevelMeter.svelte';
	import LiveActivity from './LiveActivity.svelte';
	import { languageName } from './languages';
	import { locale, t } from './i18n';
	import { micLevel, options, systemLevel } from './stores';
	import { estimateSessionCost, formatUsd } from './providers';
	import { providerDetectsLanguage } from './types';
	import type { OverlayController } from './overlayController.svelte';
	import type { SessionClock } from './sessionClock.svelte';

	/** The rail while a session runs: the setup sheet collapses to what it locked in, and what
	 *  can change mid-session — incoming audio, cost, the overlay — takes its place. */
	let {
		overlay,
		clock,
		rehearsing,
		usesMic,
		usesSystem
	}: {
		overlay: OverlayController;
		clock: SessionClock;
		/** The run is fed by the bundled sample recording rather than live audio. */
		rehearsing: boolean;
		usesMic: boolean;
		usesSystem: boolean;
	} = $props();

	// The subtitle engines detect the spoken language themselves, so there is nothing to lock.
	const roomReadsLabel = $derived(
		providerDetectsLanguage($options.provider)
			? $t.language.auto
			: languageName($options.targetLanguage, $locale)
	);
</script>

<div class="rail-head">
	<span class="rail-icon" aria-hidden="true">
		<svg
			width="13"
			height="13"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="1.8"
			stroke-linecap="round"
			><rect x="4.5" y="10.5" width="15" height="10" rx="2.5" /><path
				d="M8 10.5V8a4 4 0 0 1 8 0v2.5"
			/></svg
		>
	</span>
	<h2 class="kicker">{$t.rail.locked}</h2>
</div>

<div class="chips">
	<div class="chip">
		<span class="chip-label">{$t.rail.chip.mode}</span>
		<span class="chip-value">{$t.mode[$options.mode]}</span>
	</div>
	<div class="chip">
		<span class="chip-label">{$t.rail.chip.source}</span>
		<span class="chip-value"
			>{$options.provider === 'ondevice'
				? $t.source.demo
				: rehearsing
					? $t.source.sample
					: $t.source[$options.source]}</span
		>
	</div>
	<div class="chip">
		<span class="chip-label">{$t.rail.chip.roomReads}</span>
		<span class="chip-value">{roomReadsLabel}</span>
	</div>
	<div class="chip">
		<span class="chip-label">{$t.rail.chip.engine}</span>
		<span class="chip-value">{$t.engine[$options.provider]}</span>
	</div>
</div>

<p class="rail-note">
	<!-- The target language is fixed at session start (the backend takes it once),
	     so no mid-session F2 promise here — the idle sheet carries the F2 hint. -->
	{#if $options.provider === 'ondevice'}
		<span>{$t.rail.demoNote}</span>
	{:else if rehearsing}
		<span>{$t.rail.rehearsalNote}</span>
	{/if}
	<span>{$t.rail.lockedNote}</span>
</p>

<div class="divider"></div>

<div class="rail-section">
	<h2 class="kicker">{$t.rail.arriving}</h2>
	<LiveActivity now={clock.now} microphone={usesMic} system={usesSystem} />
	{#if usesMic}
		<LevelMeter
			level={$micLevel}
			label={$options.provider === 'ondevice' ? $t.stage.origin.demo : $t.stage.origin.microphone}
			active
		/>
	{/if}
	{#if usesSystem}
		<LevelMeter level={$systemLevel} label={$t.stage.origin.system} active />
	{/if}
</div>

<div class="cost-card">
	<div class="cost-figures">
		{#if $options.provider !== 'ondevice'}
			<div class="figure">
				<span class="chip-label">{$t.cost.estimate}</span>
				<span class="figure-value mint">
					{formatUsd(
						estimateSessionCost(
							$options.provider,
							clock.elapsedMs,
							$options.source === 'both' ? 2 : 1
						)
					)}
				</span>
			</div>
			{#if $options.source === 'both'}
				<span class="cost-tag">{$t.cost.twoSources}</span>
			{/if}
		{/if}
	</div>
	<p class="cost-note">{$t.provider.costNote[$options.provider]}</p>
</div>

<div class="divider"></div>

<div class="rail-section">
	<CaptionAppearance heading={$t.overlayControls.heading} {overlay} compact />
	<div class="overlay-actions">
		<!-- Both labels are a single verb on screen, which is all the space allows and
		     all a sighted operator needs beside the "Overlay" heading. The accessible
		     name says what is being moved or hidden, because a screen reader can arrive
		     at the button without the heading. -->
		<button
			class="tool"
			class:on={overlay.moveOverlay}
			aria-pressed={overlay.moveOverlay}
			aria-label={overlay.moveOverlay
				? $t.overlayControls.moveDoneLabel
				: $t.overlayControls.moveLabel}
			onclick={overlay.toggleMoveOverlay}
		>
			<svg
				width="13"
				height="13"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.7"
				stroke-linecap="round"
				stroke-linejoin="round"
				aria-hidden="true"
				><path
					d="M12 3.5v17M3.5 12h17M12 3.5l-3 3M12 3.5l3 3M12 20.5l-3-3M12 20.5l3-3M3.5 12l3-3M3.5 12l3 3M20.5 12l-3-3M20.5 12l-3 3"
				/></svg
			>
			{overlay.moveOverlay ? $t.overlayControls.done : $t.overlayControls.move}
		</button>
		<button
			class="tool"
			class:off={!overlay.overlayVisible}
			aria-label={overlay.overlayVisible
				? $t.overlayControls.hideLabel
				: $t.overlayControls.showLabel}
			onclick={overlay.toggleOverlayVisible}
		>
			<svg
				width="13"
				height="13"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.7"
				stroke-linecap="round"
				aria-hidden="true"
			>
				<rect x="2.5" y="4.5" width="19" height="13" rx="2" /><path d="M9 20.5h6" />
				{#if overlay.overlayVisible}<path d="M3.5 20.5l17-17" />{/if}
			</svg>
			{overlay.overlayVisible ? $t.overlayControls.hide : $t.overlayControls.show}
		</button>
	</div>
</div>

<span class="grow"></span>

<style>
	/* Direct children of the rail's scrolling column keep their height rather than
	   compress, so a short window scrolls instead of clipping text. */
	.kicker,
	.chips,
	.rail-note,
	.cost-card {
		flex: 0 0 auto;
	}

	.chips {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 8px;
	}
	.chip {
		padding: 9px 11px;
		border-radius: var(--radius-control);
		background: var(--panel-2);
		border: 1px solid var(--border-2);
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
	}
	.chip-label {
		font-size: var(--type-caption);
		font-weight: 500;
		line-height: 1;
		letter-spacing: 0.12em;
		text-transform: uppercase;
		color: var(--muted-3);
	}
	.chip-value {
		font-size: var(--type-body);
		font-weight: 500;
		line-height: 1.1;
		color: #dfe3e9;
	}
	.rail-note {
		margin: 0;
		font-size: var(--type-small);
		line-height: 1.4;
		color: var(--muted-3);
	}

	.cost-card {
		padding: 14px 15px;
		border-radius: var(--radius-card);
		background: var(--panel-2);
		border: 1px solid var(--border-2);
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.cost-figures {
		display: flex;
		align-items: baseline;
		gap: 18px;
	}
	.figure {
		display: flex;
		flex-direction: column;
		gap: 5px;
	}
	.figure-value {
		font-family: var(--font-mono);
		font-size: var(--type-heading);
		font-weight: 500;
		line-height: 1;
		color: #dfe3e9;
		font-variant-numeric: tabular-nums;
	}
	.figure-value.mint {
		color: var(--accent-soft);
	}
	.cost-tag {
		margin-left: auto;
		font-family: var(--font-mono);
		font-size: var(--type-caption);
		font-weight: 500;
		line-height: 1;
		color: var(--muted-3);
	}
	.cost-note {
		margin: 0;
		font-size: var(--type-caption);
		line-height: 1.45;
		color: var(--muted-3);
		text-wrap: pretty;
	}

	.overlay-actions {
		display: flex;
		gap: 8px;
	}
</style>
