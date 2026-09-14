<script lang="ts">
	import { fitCaptionTail } from '$lib/captionLayout';
	let {
		lead,
		text,
		interim,
		height,
		fontKey,
		language
	}: {
		lead: string;
		text: string;
		interim: boolean;
		height: number;
		fontKey: string;
		language: string;
	} = $props();
	let width = $state(0);
	let probe: HTMLParagraphElement;
	let fitted = $state('');
	const normalizedText = $derived(text.replace(/\s+/g, ' ').trim());
	const live = $derived(fitted.length <= normalizedText.length ? fitted : normalizedText);
	const prefix = $derived(
		fitted.length > normalizedText.length ? fitted.slice(0, -normalizedText.length).trimEnd() : ''
	);
	$effect(() => {
		// Font loading and resizing must refit even when no new caption arrives.
		void fontKey;
		if (!probe || width <= 0) return;
		fitted = fitCaptionTail(`${lead} ${text}`, (candidate) => {
			probe.textContent = candidate;
			if (interim) {
				const caret = document.createElement('span');
				caret.style.cssText = 'display:inline-block;width:14px;height:.86em;vertical-align:-1px';
				probe.append(caret);
			}
			return probe.getBoundingClientRect().height <= height;
		});
		probe.textContent = '';
	});
</script>

<div class="text-region" bind:clientWidth={width} style:max-height="{height}px">
	<p class="line probe" bind:this={probe} aria-hidden="true"></p>
	<!-- prettier-ignore -->
	<p class="line" lang={language} class:final={!interim}>{#if prefix}<span class="lead">{prefix}</span>{' '}{/if}{live}{#if interim && fitted}<span class="caret"></span>{/if}</p>
</div>

<style>
	.text-region {
		position: relative;
		width: 100%;
		min-width: 0;
		overflow: hidden;
	}
	.line {
		margin: 0;
		font-weight: 600;
		font-size: inherit;
		line-height: 1.34;
		letter-spacing: -0.005em;
		text-align: center;
		color: var(--caption-ink);
		overflow-wrap: anywhere;
		white-space: normal;
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
	.final {
		color: var(--caption-ink-final);
	}
	.lead {
		color: var(--caption-ink-lead);
	}
	.caret {
		display: inline-block;
		width: 4px;
		height: 0.86em;
		margin-left: 10px;
		vertical-align: -1px;
		background: #5ad1a0;
		animation: blink 1.1s steps(1) infinite;
	}
</style>
