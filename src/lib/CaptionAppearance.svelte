<script lang="ts">
	import CaptionPreview from './CaptionPreview.svelte';
	import Select from './ui/Select.svelte';
	import ToolButton from './ui/ToolButton.svelte';
	import { t, localeTag } from './i18n';
	import {
		appearance,
		overlayFontSize,
		overlayCaptionFace,
		overlayPalette,
		overlayContrast
	} from './stores';
	import { DEFAULT_APPEARANCE, sameAppearance } from './appearance';
	import { CAPTION_CONTRAST_TARGET, SCRIM_OPACITY_MIN, SCRIM_OPACITY_MAX } from './captionColour';
	import { captionFaceStack } from './captionFont';
	import type { CaptionFaceId } from './captionFont';
	import type { OverlayController } from './overlayController.svelte';
	let {
		heading,
		overlay,
		compact = false
	}: { heading: string; overlay: OverlayController; compact?: boolean } = $props();
	const idBase = $props.id();
	const contrastId = `${idBase}-contrast`;
	/** Whether anything in the overlay's appearance has been changed from what it ships with.
	 *  Drives the reset button's disabled state, so the control also answers the question
	 *  "have I changed anything?" — which is the one an operator has after an hour of
	 *  adjusting and no memory of where they started. */
	const overlayAtDefaults = $derived(sameAppearance($appearance, DEFAULT_APPEARANCE));

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
{#if !compact}<CaptionPreview {overlay} part="presets" />{/if}
<div class="appearance-layout" class:compact>
	<div class="appearance-controls">
		<div class="stepper">
			<span class="stepper-label">{$t.overlayControls.captionSize}</span>
			<button
				class="step"
				onclick={() => overlay.setFont($overlayFontSize - 2)}
				aria-label={$t.overlayControls.smaller}>−</button
			>
			<span class="stepper-value">{$overlayFontSize} px</span>
			<button
				class="step"
				onclick={() => overlay.setFont($overlayFontSize + 2)}
				aria-label={$t.overlayControls.larger}>+</button
			>
		</div>
		<label class="face-label"
			>{$t.overlayControls.captionFace}
			<Select
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
			</Select>
		</label>
		<div class="palette-options">
			{#each ['text', 'scrim'] as role}
				<div
					role="group"
					aria-label={role === 'text'
						? $t.overlayControls.captionColour
						: $t.overlayControls.scrimColour}
				>
					<span class="swatch-label"
						>{role === 'text'
							? $t.overlayControls.captionColour
							: $t.overlayControls.scrimColour}</span
					>
					<div class="swatches">
						{#each role === 'text' ? ['#ffffff', '#fff0b3', '#7fdcb6', '#b9d5ff', '#111419'] : ['#000000', '#111419', '#172b24', '#18263c', '#ffffff'] as colour}
							<button
								class="colour-choice"
								style:background={colour}
								aria-label={colour}
								aria-pressed={(role === 'text' ? $overlayPalette.text : $overlayPalette.scrim) ===
									colour}
								onclick={() => overlay.setPalette({ [role]: colour })}
							></button>
						{/each}
					</div>
				</div>
			{/each}
		</div>
		<div class="colour-row">
			<label class="swatch">
				<span class="swatch-label">{$t.overlayControls.captionColour} · {$t.design.custom}</span>
				<input
					type="color"
					value={$overlayPalette.text}
					aria-describedby={contrastId}
					oninput={(e) => overlay.setPalette({ text: e.currentTarget.value })}
				/>
			</label>
			<label class="swatch">
				<span class="swatch-label">{$t.overlayControls.scrimColour} · {$t.design.custom}</span>
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

		<ToolButton
			variant="ghost"
			size="sm"
			disabled={overlayAtDefaults}
			onclick={overlay.resetOverlayAppearance}
			aria-label={$t.overlayControls.resetLabel}
		>
			{$t.overlayControls.reset}
		</ToolButton>
	</div>
	{#if !compact}<CaptionPreview {overlay} />{/if}
</div>

<style>
	.face-label {
		display: grid;
		gap: 0.5rem;
		font-size: var(--type-small);
		color: var(--text-muted);
	}
	.appearance-layout {
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
		gap: 1.5rem;
		align-items: start;
	}
	.appearance-layout.compact {
		grid-template-columns: minmax(0, 1fr);
	}
	.appearance-controls {
		display: grid;
		gap: 0.75rem;
	}
	@media (max-width: 760px) {
		.appearance-layout {
			grid-template-columns: minmax(0, 1fr);
		}
	}
	.palette-options {
		display: grid;
		gap: 0.75rem;
	}
	.swatches {
		display: flex;
		gap: 0.5rem;
		margin-top: 0.5rem;
	}
	.colour-choice {
		width: 2rem;
		height: 2rem;
		border: 1px solid var(--line-hover);
		border-radius: var(--radius-control);
		forced-color-adjust: none;
	}
	.colour-choice[aria-pressed='true'] {
		outline: 2px solid var(--focus);
		outline-offset: 2px;
	}
	.kicker {
		flex: 0 0 auto;
	}
	.stepper {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.stepper-label {
		font-size: var(--type-small);
		line-height: 1;
		color: var(--text-muted);
		flex: 1;
	}
	.stepper-value {
		font-family: var(--font-mono);
		font-size: var(--type-body);
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
		border-radius: var(--radius-control);
		border: 1px solid var(--line-strong);
		background: var(--surface-1);
		color: var(--text-secondary);
		font-size: var(--type-label);
		font-weight: 500;
		line-height: 1;
	}
	.step:hover {
		border-color: var(--line-hover);
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
		border-radius: var(--radius-control);
		border: 1px solid var(--line-strong);
		background: var(--surface-1);
		cursor: pointer;
	}
	.swatch:hover {
		border-color: var(--line-hover);
	}
	.swatch-label {
		font-size: var(--type-small);
		line-height: 1.2;
		color: var(--text-muted);
	}
	.swatch input[type='color'] {
		flex: 0 0 auto;
		width: 26px;
		height: 20px;
		padding: 0;
		border: 1px solid var(--line-hover);
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
		font-size: var(--type-small);
		font-weight: 500;
		font-variant-numeric: tabular-nums;
		color: var(--text-secondary);
	}
	.contrast-note {
		font-size: var(--type-caption);
		line-height: 1.45;
		color: var(--text-muted);
	}
	.contrast.warn .contrast-ratio {
		color: var(--warn);
	}
	.contrast.warn .contrast-note {
		color: var(--warn-soft);
	}
	@media (forced-colors: active) {
		.swatch input[type='color'] {
			forced-color-adjust: none;
		}
	}
</style>
