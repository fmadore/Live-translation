<script lang="ts">
	import { t, localeTag } from './i18n';
	import {
		overlayFontSize,
		overlayCaptionWidth,
		overlayCaptionFace,
		overlayPalette,
		overlayContrast
	} from './stores';
	import {
		CAPTION_CONTRAST_TARGET,
		DEFAULT_CAPTION_PALETTE,
		SCRIM_OPACITY_MIN,
		SCRIM_OPACITY_MAX
	} from './captionColour';
	import { DEFAULT_OVERLAY_FONT, DEFAULT_OVERLAY_WIDTH } from './types';
	import { DEFAULT_CAPTION_FACE, captionFaceStack } from './captionFont';
	import type { CaptionFaceId } from './captionFont';
	import type { OverlayController } from './overlayController.svelte';
	let { heading, overlay }: { heading: string; overlay: OverlayController } = $props();
	const idBase = $props.id();
	const contrastId = `${idBase}-contrast`;
	/** Whether anything in the overlay's appearance has been changed from what it ships with.
	 *  Drives the reset button's disabled state, so the control also answers the question
	 *  "have I changed anything?" — which is the one an operator has after an hour of
	 *  adjusting and no memory of where they started. */
	const overlayAtDefaults = $derived(
		$overlayFontSize === DEFAULT_OVERLAY_FONT &&
			$overlayCaptionWidth === DEFAULT_OVERLAY_WIDTH &&
			$overlayCaptionFace === DEFAULT_CAPTION_FACE &&
			$overlayPalette.text === DEFAULT_CAPTION_PALETTE.text &&
			$overlayPalette.scrim === DEFAULT_CAPTION_PALETTE.scrim &&
			$overlayPalette.scrimOpacity === DEFAULT_CAPTION_PALETTE.scrimOpacity
	);

	/** The achieved ratio, written the way the interface language writes numbers — 5.7 in
	 *  English, 5,7 in French. */
	const contrastReading = $derived(
		$overlayContrast.worst.toLocaleString($localeTag, {
			minimumFractionDigits: 1,
			maximumFractionDigits: 1
		})
	);
	const contrastTarget = $derived(
		CAPTION_CONTRAST_TARGET.toLocaleString($localeTag, {
			minimumFractionDigits: 1,
			maximumFractionDigits: 1
		})
	);
</script>

<!-- Rendered in two places on purpose. In the running rail, because the size is what gets
	     nudged mid-session when someone at the back cannot read; and in the settings panel,
	     because the whole look is chosen before a room fills, and an operator should not have
	     to start a session — or pay for one — to choose a typeface. One component over one set
	     of stores, so the two views cannot disagree about what the overlay is wearing.

	     The heading is a parameter rather than fixed: in the rail this sits under "Overlay"
	     among the live controls, while in the panel it names itself against the other
	     preferences. -->
<h2 class="kicker">{heading}</h2>
<div class="stepper">
	<span class="stepper-label">{$t.overlayControls.captionSize}</span>
	<button
		class="step"
		onclick={() => overlay.setFont($overlayFontSize - 2)}
		aria-label={$t.overlayControls.smaller}>−</button
	>
	<span class="stepper-value">{$overlayFontSize}</span>
	<button
		class="step"
		onclick={() => overlay.setFont($overlayFontSize + 2)}
		aria-label={$t.overlayControls.larger}>+</button
	>
</div>
<div class="stepper">
	<span class="stepper-label">{$t.overlayControls.captionWidth}</span>
	<button
		class="step"
		onclick={() => overlay.setCaptionWidth($overlayCaptionWidth - 2)}
		aria-label={$t.overlayControls.narrower}>−</button
	>
	<span class="stepper-value">{$overlayCaptionWidth}</span>
	<button
		class="step"
		onclick={() => overlay.setCaptionWidth($overlayCaptionWidth + 2)}
		aria-label={$t.overlayControls.wider}>+</button
	>
</div>
<div class="select-row">
	<select
		aria-label={$t.overlayControls.captionFace}
		value={$overlayCaptionFace}
		onchange={(e) => overlay.setCaptionFace(e.currentTarget.value as CaptionFaceId)}
	>
		<!-- Each option is set in the face it names, so the list is its own preview.
			     The names are proper nouns and stay untranslated; only the note on the
			     bundled default says anything, and it is the one thing that needs to. -->
		{#each overlay.captionFaces as face (face.id)}
			<option value={face.id} style="font-family: {captionFaceStack(face.id)}">
				{face.bundled ? $t.overlayControls.faceDefault(face.label) : face.label}
			</option>
		{/each}
	</select>
	<svg
		class="chevron"
		width="12"
		height="12"
		viewBox="0 0 24 24"
		fill="none"
		stroke="currentColor"
		stroke-width="2"
		stroke-linecap="round"
		aria-hidden="true"><path d="M6 9.5l6 6 6-6" /></svg
	>
</div>
<div class="colour-row">
	<label class="swatch">
		<span class="swatch-label">{$t.overlayControls.captionColour}</span>
		<input
			type="color"
			value={$overlayPalette.text}
			aria-describedby={contrastId}
			oninput={(e) => overlay.setPalette({ text: e.currentTarget.value })}
		/>
	</label>
	<label class="swatch">
		<span class="swatch-label">{$t.overlayControls.scrimColour}</span>
		<input
			type="color"
			value={$overlayPalette.scrim}
			aria-describedby={contrastId}
			oninput={(e) => overlay.setPalette({ scrim: e.currentTarget.value })}
		/>
	</label>
</div>
<div class="stepper">
	<span class="stepper-label">{$t.overlayControls.scrimOpacity}</span>
	<button
		class="step"
		disabled={$overlayPalette.scrimOpacity <= SCRIM_OPACITY_MIN}
		onclick={() => overlay.setPalette({ scrimOpacity: $overlayPalette.scrimOpacity - 0.05 })}
		aria-label={$t.overlayControls.weakerScrim}>−</button
	>
	<span class="stepper-value">{Math.round($overlayPalette.scrimOpacity * 100)}%</span>
	<button
		class="step"
		disabled={$overlayPalette.scrimOpacity >= SCRIM_OPACITY_MAX}
		onclick={() => overlay.setPalette({ scrimOpacity: $overlayPalette.scrimOpacity + 0.05 })}
		aria-label={$t.overlayControls.strongerScrim}>+</button
	>
</div>
<!-- Not a live region on purpose: this changes on every step of a colour drag, and
	     `docs/accessibility.md` keeps announcements for things worth interrupting a
	     reader for. It is the description of the controls instead, so it is read on
	     arrival at the one moment it is worth hearing. -->
<p class="contrast" class:warn={!$overlayContrast.passes} id={contrastId}>
	<span class="contrast-ratio">{$t.overlayControls.contrast(contrastReading)}</span>
	<span class="contrast-note">
		{$overlayContrast.passes
			? $t.overlayControls.contrastOk
			: $t.overlayControls.contrastLow(
					$t.overlayControls.contrastStep[$overlayContrast.worstStep],
					contrastTarget
				)}
	</span>
</p>
<button
	class="reset"
	disabled={overlayAtDefaults}
	onclick={overlay.resetOverlayAppearance}
	aria-label={$t.overlayControls.resetLabel}
>
	{$t.overlayControls.reset}
</button>

<style>
	.kicker {
		flex: 0 0 auto;
	}
	.kicker {
		margin: 0;
		font-size: var(--type-10-5);
		font-weight: 600;
		line-height: 1;
		letter-spacing: 0.15em;
		text-transform: uppercase;
		color: var(--muted-2);
	}
	.select-row {
		position: relative;
		display: flex;
		align-items: center;
		padding: 0.625rem 0.75rem;
		border-radius: 9px;
		border: 1px solid var(--border);
		background: var(--panel-2);
		margin-top: 2px;
	}
	.select-row select {
		appearance: none;
		width: 100%;
		border: 0;
		background: transparent;
		color: var(--text-soft);
		font-size: var(--type-12-5);
		line-height: 1;
		padding: 0 20px 0 0;
	}
	.select-row select:focus-visible {
		outline: 2px solid var(--accent-border);
		outline-offset: 4px;
		border-radius: 3px;
	}
	.select-row select option {
		background: var(--panel-2);
		color: var(--text);
	}
	.chevron {
		position: absolute;
		right: 12px;
		color: var(--muted-2);
		pointer-events: none;
	}
	.stepper {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.stepper-label {
		font-size: var(--type-12);
		line-height: 1;
		color: var(--muted);
		flex: 1;
	}
	.stepper-value {
		font-family: var(--font-mono);
		font-size: var(--type-13);
		font-weight: 500;
		line-height: 1;
		/* Two mono digits, so the buttons either side stop moving as the number changes. */
		min-width: 2ch;
		text-align: center;
		font-variant-numeric: tabular-nums;
	}
	.step {
		width: 30px;
		height: 30px;
		border-radius: 8px;
		border: 1px solid var(--border);
		background: var(--panel-2);
		color: var(--text-soft);
		font-size: var(--type-14);
		font-weight: 500;
		line-height: 1;
	}
	.step:hover {
		border-color: var(--border-hover);
	}
	.colour-row {
		display: grid;
		grid-auto-flow: column;
		grid-auto-columns: 1fr;
		gap: 0.5rem;
	}
	.swatch {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.5rem;
		padding: 0.5rem 0.625rem;
		border-radius: 9px;
		border: 1px solid var(--border);
		background: var(--panel-2);
		cursor: pointer;
	}
	.swatch:hover {
		border-color: var(--border-hover);
	}
	.swatch-label {
		font-size: var(--type-12);
		line-height: 1.2;
		color: var(--muted);
	}
	.swatch input[type='color'] {
		flex: 0 0 auto;
		width: 26px;
		height: 20px;
		padding: 0;
		border: 1px solid var(--border-hover);
		border-radius: 5px;
		background: none;
		cursor: pointer;
	}
	.swatch input[type='color']::-webkit-color-swatch-wrapper {
		padding: 0;
	}
	.swatch input[type='color']::-webkit-color-swatch {
		border: none;
		border-radius: 4px;
	}
	.contrast {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 0.375rem;
		margin: 0;
	}
	.contrast-ratio {
		font-family: var(--font-mono);
		font-size: var(--type-12);
		font-weight: 500;
		font-variant-numeric: tabular-nums;
		color: var(--text-dim);
	}
	.contrast-note {
		font-size: var(--type-11);
		line-height: 1.45;
		color: var(--muted-2);
	}
	.contrast.warn .contrast-ratio {
		color: var(--warn);
	}
	.contrast.warn .contrast-note {
		color: var(--warn-soft);
	}
	.reset {
		align-self: flex-start;
		padding: 0.375rem 0.625rem;
		border-radius: 8px;
		border: 1px solid var(--border);
		background: transparent;
		color: var(--muted);
		font-size: var(--type-11);
		font-weight: 500;
		line-height: 1;
	}
	.reset:hover:not(:disabled) {
		border-color: var(--border-hover);
		color: var(--text-soft);
	}
	.reset:disabled {
		opacity: 0.45;
		cursor: default;
	}
	@media (forced-colors: active) {
		.swatch input[type='color'] {
			forced-color-adjust: none;
		}
	}
</style>
