<script lang="ts">
	import { api } from './tauri';
	import { clearTranscript as clearTranscriptStore } from './stores';
	import { formatTranscript, groupTranscript, transcriptFilename, type TranscriptFormat } from './transcript';
	import type { Origin, OutputMode, TranscriptLine } from './types';

	interface Props {
		mode: OutputMode;
		transcript: TranscriptLine[];
		onError: (message: string) => void;
	}

	let { mode, transcript, onError }: Props = $props();
	let savedPath = $state('');

	// The log names the two sides of the room, not the devices — the device names
	// (`ORIGIN_LABEL`) are what the saved file uses.
	const SIDE_LABEL: Record<Origin, string> = { microphone: 'Room', system: 'Remote' };

	// Show the log exactly as it will be saved: chronological, grouped into a paragraph per
	// audio source, newest at the bottom.
	const paragraphs = $derived(groupTranscript(transcript));

	let logEl = $state<HTMLUListElement | null>(null);

	$effect(() => {
		// Re-runs whenever the log grows so the newest paragraph stays in view.
		paragraphs;
		if (logEl) logEl.scrollTop = logEl.scrollHeight;
	});

	async function save(format: TranscriptFormat) {
		if (!transcript.length) return;
		const now = new Date();
		try {
			savedPath = await api.saveTranscript(
				formatTranscript(transcript, format, now),
				transcriptFilename(now, format)
			);
		} catch (error) {
			onError(String(error));
		}
	}

	function clear() {
		clearTranscriptStore();
		savedPath = '';
	}
</script>

<section class="monitor">
	<div class="head">
		<span class="kicker">Transcript</span>
		<span class="count">{transcript.length} {transcript.length === 1 ? 'line' : 'lines'}</span>
		<div class="spacer"></div>
		<button class="ghost" disabled={!transcript.length} onclick={() => save('text')}>Save text</button>
		<button class="ghost" disabled={!transcript.length} onclick={() => save('markdown')}>Save Markdown</button>
		<button class="ghost quiet" disabled={!transcript.length} onclick={clear}>Clear</button>
	</div>

	{#if savedPath}
		<p class="saved">Saved to <code>{savedPath}</code></p>
	{/if}

	{#if paragraphs.length}
		<ul class="log" bind:this={logEl}>
			{#each paragraphs as paragraph (paragraph.id)}
				<li class="origin-{paragraph.origin}">
					<span class="side">{SIDE_LABEL[paragraph.origin]}</span>
					<span class="text">{paragraph.text}</span>
				</li>
			{/each}
		</ul>
	{:else}
		<p class="hint">
			{mode === 'translate'
				? 'Finalized translations collect here, ready to save as text or Markdown.'
				: 'Finalized subtitles collect here, ready to save as text or Markdown.'}
		</p>
	{/if}
</section>

<style>
	.monitor {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin-top: 22px;
		padding-top: 18px;
		border-top: 1px solid var(--hairline);
		/* The stage is a scrolling flex column; keep the block at its natural height there. */
		flex: 0 0 auto;
	}
	.head {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.kicker {
		font-size: 10.5px;
		font-weight: 600;
		line-height: 1;
		letter-spacing: 0.15em;
		text-transform: uppercase;
		color: var(--muted-2);
	}
	.count {
		font-family: var(--font-mono);
		font-size: 11px;
		line-height: 1;
		color: var(--muted-3);
		font-variant-numeric: tabular-nums;
	}
	.spacer {
		flex: 1;
	}
	button.ghost {
		font-size: 11.5px;
		font-weight: 500;
		line-height: 1;
		color: var(--text-soft);
		padding: 7px 11px;
		border-radius: 7px;
		border: 1px solid var(--border);
		background: transparent;
	}
	button.ghost:hover:not(:disabled) {
		border-color: var(--border-hover);
		color: var(--text);
	}
	button.ghost.quiet {
		color: var(--muted-2);
		border-color: transparent;
	}
	button.ghost.quiet:hover:not(:disabled) {
		border-color: transparent;
		color: var(--text);
	}
	.saved {
		margin: 0;
		font-size: 12px;
		color: var(--accent-soft);
		word-break: break-all;
	}
	.saved code {
		font-family: var(--font-mono);
		font-size: 11px;
	}
	.hint {
		margin: 0;
		font-size: 12.5px;
		line-height: 1.55;
		color: var(--muted-3);
	}
	.log {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 10px;
		max-height: 180px;
		overflow-y: auto;
	}
	.log li {
		display: grid;
		grid-template-columns: 68px 1fr;
		gap: 14px;
	}
	.side {
		font-size: 9.5px;
		font-weight: 500;
		line-height: 1.6;
		letter-spacing: 0.12em;
		text-transform: uppercase;
	}
	.origin-microphone .side {
		color: var(--faint);
	}
	/* Dimmer than --room-soft: in the log the label is a marker, not a heading. */
	.origin-system .side {
		color: #6b8fd6;
	}
	.text {
		font-size: 12.5px;
		line-height: 1.55;
		color: var(--muted);
		text-wrap: pretty;
	}
</style>
