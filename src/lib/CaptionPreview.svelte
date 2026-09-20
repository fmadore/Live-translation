<script lang="ts">
	import { t } from './i18n';
	import {
		overlayCaptionLayout,
		overlayFontSize,
		overlayPalette,
		overlayCaptionFace
	} from './stores';
	import { captionCssVars, DEFAULT_CAPTION_PALETTE } from './captionColour';
	import { captionFaceStack } from './captionFont';
	import type { OverlayController } from './overlayController.svelte';
	let { overlay, part = 'preview' }: { overlay: OverlayController; part?: 'presets' | 'preview' } =
		$props();
	const vars = $derived(
		Object.entries(captionCssVars($overlayPalette))
			.map(([k, v]) => `${k}:${v}`)
			.join(';')
	);
	function matches(kind: 'standard' | 'projector' | 'contrast') {
		return (
			$overlayFontSize === (kind === 'projector' ? 52 : 38) &&
			$overlayCaptionFace === 'archivo' &&
			$overlayCaptionLayout === (kind === 'projector' ? 'stable' : 'fit') &&
			$overlayPalette.text === '#ffffff' &&
			$overlayPalette.scrim === (kind === 'standard' ? DEFAULT_CAPTION_PALETTE.scrim : '#000000') &&
			$overlayPalette.scrimOpacity ===
				(kind === 'standard'
					? DEFAULT_CAPTION_PALETTE.scrimOpacity
					: kind === 'contrast'
						? 1
						: 0.85)
		);
	}

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

{#if part === 'presets'}
	<div class="presets" role="group" aria-label={$t.usability.preset}>
		<button aria-pressed={matches('standard')} onclick={() => preset('standard')}
			>{$t.usability.standard}</button
		>
		<button aria-pressed={matches('projector')} onclick={() => preset('projector')}
			>{$t.usability.projector}</button
		>
		<button aria-pressed={matches('contrast')} onclick={() => preset('contrast')}
			>{$t.usability.contrast}</button
		>
	</div>
{:else}
	<div class="preview-section">
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
	</div>
{/if}

<style>
	.presets button[aria-pressed='true'] {
		border-color: var(--accent-border);
		color: var(--accent-soft);
		background: var(--accent-bg);
	}
	@media (forced-colors: active) {
		.presets button[aria-pressed='true'] {
			outline: 2px solid Highlight;
		}
	}
	.presets {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
	}
	.presets button {
		flex: 1;
		padding: 0.625rem;
		border: 1px solid var(--border);
		border-radius: var(--radius-control);
		background: var(--panel-2);
		color: var(--text-soft);
		font-size: var(--type-12);
	}
	.presets button:hover {
		border-color: var(--accent-border);
		color: var(--accent-soft);
	}
	p {
		font-size: var(--type-12);
		line-height: 1.5;
		color: var(--muted);
	}
	.preview {
		padding: 1rem;
	}
	.preview {
		background: #111;
		overflow: hidden;
		border-radius: var(--radius-control);
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
