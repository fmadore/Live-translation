<script lang="ts">
	import { fitNoticeFont } from '#lib/captionLayout.js';

	/** The operator's notice in place of the captions: one centred line, as large as a caption
	 *  row would be and smaller when the strip is too shallow or the text too long for it, so
	 *  it is read whole at any height. Plain text: it is never parsed as markup. */
	let {
		text,
		maxFont,
		height,
		fontKey
	}: {
		text: string;
		/** The size a single caption row would be drawn at in this strip (`fitRows`). */
		maxFont: number;
		/** The height the notice has, in px. */
		height: number;
		/** Changes when the face or its loading does, so the size is measured again. */
		fontKey: string;
	} = $props();

	let probe: HTMLParagraphElement;
	let width = $state(0);
	let font = $state(0);

	$effect(() => {
		void fontKey;
		if (!probe || width <= 0 || height <= 0 || !text) return;
		probe.textContent = text;
		font = fitNoticeFont(maxFont, (size) => {
			probe.style.fontSize = `${size}px`;
			return probe.getBoundingClientRect().height <= height;
		});
		probe.textContent = '';
	});
</script>

<div class="notice-region" bind:clientWidth={width} style:max-height="{height}px">
	<p class="line probe" bind:this={probe} aria-hidden="true"></p>
	<!-- The text's language is whatever the operator typed it in: none is claimed, and the
	     direction is left to the text. -->
	<p class="line" lang="" dir="auto" style:font-size={font ? `${font}px` : undefined}>{text}</p>
</div>

<style>
	.notice-region {
		position: relative;
		width: 100%;
		min-width: 0;
		overflow: hidden;
	}
	/* Set like a finished caption, so the room reads it as the overlay's own voice. */
	.line {
		margin: 0;
		font-weight: 600;
		line-height: 1.34;
		letter-spacing: -0.005em;
		text-align: center;
		color: var(--caption-ink-final);
		overflow-wrap: anywhere;
		/* Sized by measurement, at once. Reduced motion gives every property a 0.01 ms transition
		   (`app.css`): the probe then reads the size it had, not the one just set, and the line
		   can be left drawn at its old size. */
		transition: none;
		text-shadow:
			0 1px 3px var(--caption-halo-tight),
			0 2px 14px var(--caption-halo-soft);
	}
	.probe {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		visibility: hidden;
		pointer-events: none;
	}
</style>
