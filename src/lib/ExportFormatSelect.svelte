<script lang="ts">
	import Select from './ui/Select.svelte';
	import { t } from './i18n';
	import { exportFormat, TRANSCRIPT_FORMATS } from './exportFormat';
	import type { TranscriptFormat } from './transcript';

	/** The transcript's export format: the one remembered preference (`exportFormat.ts`), chosen
	 *  in the same words wherever a transcript is saved. */
	let {
		disabled = false,
		compact = false
	}: {
		disabled?: boolean;
		/** The transcript toolbar's small select, beside its other small controls, rather than
		 *  the form field the history view lays out with its buttons. */
		compact?: boolean;
	} = $props();

	// The formats' own names are not translated; only "plain text" is a description.
	const labels = $derived<Record<TranscriptFormat, string>>({
		markdown: 'Markdown (.md)',
		text: `${$t.transcript.plainText} (.txt)`,
		vtt: 'WebVTT (.vtt)',
		srt: 'SubRip (.srt)'
	});
</script>

{#snippet formats()}
	{#each TRANSCRIPT_FORMATS as format (format)}
		<option value={format}>{labels[format]}</option>
	{/each}
{/snippet}

{#if compact}
	<select aria-label={$t.transcript.format} bind:value={$exportFormat} {disabled}>
		{@render formats()}
	</select>
{:else}
	<Select aria-label={$t.transcript.format} bind:value={$exportFormat} {disabled}>
		{@render formats()}
	</Select>
{/if}

<style>
	/* The transcript toolbar's small select, drawn as the copy-window select beside it. */
	select {
		font: inherit;
		min-height: var(--control-sm);
		font-size: var(--type-small);
		color: var(--text-body);
		background: var(--surface-0);
		border: 1px solid var(--line-strong);
		border-radius: var(--radius-control);
		padding: var(--space-1);
		max-width: 100%;
	}
</style>
