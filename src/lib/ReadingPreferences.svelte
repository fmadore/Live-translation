<script lang="ts">
	import { t } from './i18n';
	import {
		overlayHoldSeconds,
		overlayPace,
		overlayCaptionLayout,
		overlayFontSize,
		overlayPalette,
		overlayCaptionFace
	} from './stores';
	import { captionCssVars, DEFAULT_CAPTION_PALETTE } from './captionColour';
	import { captionFaceStack } from './captionFont';
	import type { OverlayController } from './overlayController.svelte';
	let { overlay }: { overlay: OverlayController } = $props();
	const vars = $derived(
		Object.entries(captionCssVars($overlayPalette))
			.map(([k, v]) => `${k}:${v}`)
			.join(';')
	);
	function preset(kind: 'standard' | 'projector' | 'contrast') {
		overlay.setPalette({
			text: '#ffffff',
			scrim: kind === 'standard' ? DEFAULT_CAPTION_PALETTE.scrim : '#000000',
			scrimOpacity:
				kind === 'standard' ? DEFAULT_CAPTION_PALETTE.scrimOpacity : kind === 'contrast' ? 1 : 0.85
		});
		overlay.setFont(kind === 'projector' ? 52 : 38);
		overlay.setCaptionFace('archivo');
		overlay.setCaptionLayout(kind === 'projector' ? 'stable' : 'fit');
	}
</script>

<div class="reading">
	<label
		>{$t.usability.hold}
		<input
			type="number"
			min="2"
			max="30"
			step="1"
			value={$overlayHoldSeconds}
			disabled={$overlayCaptionLayout === 'stable'}
			onchange={(e) => overlay.setHoldSeconds(e.currentTarget.valueAsNumber)}
		/>
	</label>
	<p>{$t.usability.holdHint}</p>
	<label
		>{$t.usability.pace}
		<select
			value={$overlayPace}
			onchange={(e) => overlay.setPace(e.currentTarget.value === 'steady' ? 'steady' : 'immediate')}
		>
			<option value="immediate">{$t.usability.immediate}</option><option value="steady"
				>{$t.usability.steady}</option
			>
		</select>
	</label>
	<p>{$t.usability.paceHint}</p>
	<details>
		<summary>{$t.usability.preview}</summary>
		<div class="presets" role="group" aria-label={$t.usability.preset}>
			<button onclick={() => preset('standard')}>{$t.usability.standard}</button>
			<button onclick={() => preset('projector')}>{$t.usability.projector}</button>
			<button onclick={() => preset('contrast')}>{$t.usability.contrast}</button>
		</div>
		<p>{$t.usability.previewHint}</p>
		{#each ['bright', 'dark'] as background}
			<p>{background === 'bright' ? $t.usability.bright : $t.usability.dark}</p>
			<div class="preview" class:bright={background === 'bright'} style={vars}>
				<div
					class="sample"
					style:font-family={captionFaceStack($overlayCaptionFace)}
					style:font-size="{$overlayFontSize}px"
					style:text-align={$overlayCaptionLayout === 'stable' ? 'left' : 'center'}
				>
					{$t.usability.sample}
				</div>
			</div>
		{/each}
	</details>
</div>

<style>
	.reading {
		margin: 1rem 0;
		font-size: var(--type-12);
	}
	label {
		display: grid;
		gap: 0.5rem;
		margin-top: 0.75rem;
	}
	input,
	select,
	button {
		font: inherit;
		color: var(--text);
		background: var(--panel-2);
		border: 1px solid var(--border);
		border-radius: 6px;
		padding: 0.5rem;
		min-width: 0;
	}
	input:disabled {
		opacity: 0.5;
	}
	p {
		color: var(--muted);
		line-height: 1.5;
	}
	summary {
		cursor: pointer;
		padding: 0.5rem 0;
	}
	.presets {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
		margin: 0.75rem 0;
	}
	.preview {
		background: #111;
		overflow: hidden;
		border-radius: 6px;
	}
	.preview.bright {
		background: #fff;
	}
	.sample {
		color: var(--caption-ink);
		background: var(--caption-scrim-strong);
		padding: 12px;
		line-height: 1.34;
		font-weight: 600;
		overflow-wrap: anywhere;
		text-shadow: 0 1px 3px var(--caption-halo-tight);
	}
</style>
