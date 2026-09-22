<script lang="ts">
	import { captionDirection } from '$lib/languages';
	import { fitCaptionTail } from '$lib/captionLayout';
	let {
		stable = false,
		lead,
		text,
		interim,
		height,
		fontKey,
		language
	}: {
		stable?: boolean;
		lead: string;
		text: string;
		interim: boolean;
		height: number;
		fontKey: string;
		language: string;
	} = $props();
	let fullHeight = $state(0);
	let lineHeight = $state(1);
	const visibleRows = $derived(Math.max(0, Math.floor(height / lineHeight)));
	const stableHeight = $derived(visibleRows * lineHeight);
	const scrollOffset = $derived(
		Math.max(0, Math.round(fullHeight / lineHeight) - visibleRows) * lineHeight
	);
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
		void language;
		if (!probe || width <= 0) return;
		lineHeight = parseFloat(getComputedStyle(probe).lineHeight) || 1;
		if (stable) return;
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

<div
	dir={captionDirection(language)}
	lang={language}
	class="text-region"
	class:stable
	bind:clientWidth={width}
	style:max-height="{height}px"
>
	<p class="line probe" bind:this={probe} aria-hidden="true"></p>
	{#if stable}
		<div class="stable-viewport" style:height="{stableHeight}px">
			<p
				class="line"
				lang={language}
				bind:clientHeight={fullHeight}
				style:transform="translateY(-{scrollOffset}px)"
			>
				{lead}{lead && text ? ' ' : ''}{text}
			</p>
		</div>
	{:else}
		<!-- prettier-ignore -->
		<p class="line" lang={language} class:final={!interim}>{#if prefix}<span class="lead">{prefix}</span>{' '}{/if}{live}{#if interim && fitted}<span class="caret"></span>{/if}</p>
	{/if}
</div>

<style>
	.stable {
		align-self: flex-start;
	}
	.stable .line {
		text-align: start;
	}
	.stable-viewport {
		overflow: hidden;
	}
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
		margin-inline-start: 10px;
		vertical-align: -1px;
		background: #5ad1a0;
		animation: blink 1.1s steps(1) infinite;
	}
</style>
