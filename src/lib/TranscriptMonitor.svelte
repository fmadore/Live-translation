<script lang="ts">
	import { api } from './tauri';
	import { clearTranscript as clearTranscriptStore } from './stores';
	import {
		formatTranscript,
		groupTranscript,
		transcriptFilename,
		ORIGIN_LABEL,
		type TranscriptFormat
	} from './transcript';
	import type { Caption, OutputMode, TranscriptLine } from './types';

	interface Props {
		mode: OutputMode;
		latestCaption: Caption | null;
		transcript: TranscriptLine[];
		onError: (message: string) => void;
	}

	let { mode, latestCaption, transcript, onError }: Props = $props();
	let savedPath = $state('');

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

<section class="panel monitor">
	<div class="monitor-head">
		<h2>Live monitor</h2>
		<div class="monitor-actions">
			<button class="ghost" disabled={!transcript.length} onclick={() => save('text')}>Save text</button>
			<button class="ghost" disabled={!transcript.length} onclick={() => save('markdown')}>Save Markdown</button>
			<button class="ghost" disabled={!transcript.length} onclick={clear}>Clear</button>
		</div>
	</div>
	{#if savedPath}
		<p class="saved">Saved to <code>{savedPath}</code></p>
	{/if}
	{#if latestCaption}
		<div class="current">
			{#if latestCaption.sourceText}<div class="src">{latestCaption.sourceText}</div>{/if}
			<div class="trans" class:interim={!latestCaption.final}>{latestCaption.text}</div>
		</div>
	{:else}
		<p class="hint">
			{mode === 'translate'
				? 'Translated captions will appear here and on the overlay.'
				: 'Live subtitles will appear here and on the overlay.'}
		</p>
	{/if}

	{#if paragraphs.length}
		<ul class="log" bind:this={logEl}>
			{#each paragraphs as paragraph (paragraph.id)}
				<li>
					<span class="log-origin">{ORIGIN_LABEL[paragraph.origin]}</span>
					{paragraph.text}
				</li>
			{/each}
		</ul>
	{/if}
</section>

<style>
	.panel { background: var(--panel); border: 1px solid var(--border); border-radius: 12px; padding: 16px; }
	h2 { margin: 0 0 10px; font-size: 14px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--muted); }
	.monitor-head { display: flex; align-items: center; justify-content: space-between; }
	.monitor-actions { display: flex; gap: 8px; }
	button.ghost { background: transparent; color: var(--text); border: 1px solid var(--border); border-radius: 8px; padding: 9px 14px; }
	.saved { font-size: 12px; color: var(--accent-2); margin: 0 0 10px; word-break: break-all; }
	.saved code { background: var(--panel-2); padding: 1px 5px; border-radius: 4px; }
	.current { display: flex; flex-direction: column; gap: 6px; }
	.current .src { color: var(--muted); font-size: 14px; }
	.current .trans { font-size: 22px; line-height: 1.3; }
	.current .trans.interim { opacity: 0.65; font-style: italic; }
	.hint { color: var(--muted); font-size: 13px; }
	.log { list-style: none; margin: 14px 0 0; padding: 0; max-height: 180px; overflow-y: auto; border-top: 1px solid var(--border); }
	.log li { padding: 7px 0; border-bottom: 1px solid var(--border); font-size: 14px; color: var(--muted); }
	.log-origin { display: block; font-size: 11px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--accent-2); }
</style>
