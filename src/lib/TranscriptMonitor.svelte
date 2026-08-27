<script lang="ts">
	import { api, isTauri } from './tauri';
	import { t } from './i18n';
	import { asStatus, type AppError } from './errors';
	import {
		clearTranscript as clearTranscriptStore,
		recoveryEnabled,
		savedPath,
		transcriptDirty
	} from './stores';
	import { TRANSCRIPT_WARN_LINES } from './document';
	import { saveTranscriptDocument } from './saveDocument';
	import { groupTranscript, type TranscriptFormat } from './transcript';
	import type { Origin, OutputMode, TranscriptLine } from './types';

	interface Props {
		mode: OutputMode;
		transcript: TranscriptLine[];
		onError: (message: string | AppError) => void;
	}

	let { mode, transcript, onError }: Props = $props();
	// A browser preview has no filesystem to spool to, and inviting the operator to switch on a
	// feature whose only outcome would be an IPC error is the mistake issue #29 fixed elsewhere.
	const desktop = isTauri();
	let saving = $state(false);
	// Clear throws away unsaved text, so when there is any it asks once rather than acting on
	// the first click.
	let confirmingClear = $state(false);

	// The log names the two sides of the room; the saved file names the devices
	// (`export.origin`). Both follow the interface language.
	const sideLabel = $derived<Record<Origin, string>>($t.transcript.side);

	// Show the log exactly as it will be saved: chronological, grouped into a paragraph per
	// audio source, newest at the bottom.
	const paragraphs = $derived(groupTranscript(transcript));

	// What a screen reader hears after a save. Empty while there is nothing new to report, so
	// the region stays silent between saves.
	const saveAnnouncement = $derived(
		$savedPath && !$transcriptDirty ? $t.transcript.savedAnnouncement($savedPath) : ''
	);

	// Nothing is ever dropped (issue #25), but a log this long is a session worth putting on
	// disk before something else decides for the operator.
	const veryLong = $derived(transcript.length >= TRANSCRIPT_WARN_LINES);

	let logEl = $state<HTMLUListElement | null>(null);

	$effect(() => {
		// Re-runs whenever the log grows so the newest paragraph stays in view.
		paragraphs;
		if (logEl) logEl.scrollTop = logEl.scrollHeight;
	});

	// New captions withdraw the pending "discard?" question: the log the operator was about to
	// throw away is no longer the log in front of them.
	$effect(() => {
		transcript.length;
		confirmingClear = false;
	});

	async function save(format: TranscriptFormat) {
		if (!transcript.length || saving) return;
		saving = true;
		try {
			await saveTranscriptDocument(format);
		} catch (error) {
			onError(asStatus(error));
		} finally {
			saving = false;
		}
	}

	function clear() {
		if ($transcriptDirty && !confirmingClear) {
			confirmingClear = true;
			return;
		}
		confirmingClear = false;
		clearTranscriptStore();
		// The spool covered exactly the text just discarded, so it goes with it.
		if (desktop) void api.clearRecovery().catch(() => {});
	}

	function toggleRecovery(enabled: boolean) {
		if (!desktop) return;
		recoveryEnabled.set(enabled);
		// Switching it off has to remove what it already wrote, or "disabled" would still
		// leave captions sitting on disk.
		if (!enabled) void api.clearRecovery().catch((error) => onError(asStatus(error)));
	}
</script>

<section class="monitor">
	<div class="head">
		<h2 class="kicker">{$t.transcript.heading}</h2>
		<span class="count">{$t.transcript.lines(transcript.length)}</span>
		<!-- The whole point of the badge: on screen, a transcript that exists only in memory
		     looks exactly like one that is on disk. -->
		{#if transcript.length}
			<span class="state" class:unsaved={$transcriptDirty} data-testid="save-state">
				{$transcriptDirty ? $t.transcript.unsaved : $t.transcript.saved}
			</span>
		{/if}
		<div class="spacer"></div>
		<button
			class="ghost"
			disabled={!transcript.length || saving}
			aria-busy={saving}
			onclick={() => save('text')}
		>
			{$t.transcript.saveText}
		</button>
		<button
			class="ghost"
			disabled={!transcript.length || saving}
			aria-busy={saving}
			onclick={() => save('markdown')}
		>
			{$t.transcript.saveMarkdown}
		</button>
		<button
			class="ghost quiet"
			class:confirming={confirmingClear}
			disabled={!transcript.length || saving}
			onclick={clear}
		>
			{confirmingClear ? $t.transcript.confirmClear : $t.transcript.clear}
		</button>
	</div>

	<!-- The announcement lives in its own always-present region — a live region created at the
	     same moment as its text is routinely missed, and this one is out of flow, so it costs
	     the layout nothing between saves. The visible lines below repeat it for the eye. -->
	<p class="sr-only" role="status">{saveAnnouncement}</p>
	{#if $savedPath && !$transcriptDirty}
		<p class="saved" aria-hidden="true">{$t.transcript.savedTo} <code>{$savedPath}</code></p>
	{:else if $savedPath}
		<p class="hint" aria-hidden="true">
			{$t.transcript.staleBefore} <code>{$savedPath}</code>
			{$t.transcript.staleAfter}
		</p>
	{/if}

	{#if veryLong && $transcriptDirty}
		<p class="warn" role="status">{$t.transcript.longSession(TRANSCRIPT_WARN_LINES)}</p>
	{/if}

	{#if paragraphs.length}
		<ul class="log" bind:this={logEl}>
			{#each paragraphs as paragraph (paragraph.id)}
				<li class="origin-{paragraph.origin}">
					<span class="side">{sideLabel[paragraph.origin]}</span>
					<span class="text">{paragraph.text}</span>
				</li>
			{/each}
		</ul>
	{:else}
		<p class="hint">
			{mode === 'translate' ? $t.transcript.emptyTranslate : $t.transcript.emptySubtitles}
		</p>
	{/if}

	<label class="recovery">
		<input
			type="checkbox"
			checked={$recoveryEnabled}
			disabled={!desktop}
			onchange={(e) => toggleRecovery(e.currentTarget.checked)}
		/>
		<span>
			<span class="recovery-title">{$t.transcript.recovery.title}</span>
			<span class="recovery-note">
				{desktop ? $t.transcript.recovery.note : $t.transcript.recovery.needsDesktop}
			</span>
		</span>
	</label>
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
		margin: 0;
		font-size: var(--type-10-5);
		font-weight: 600;
		line-height: 1;
		letter-spacing: 0.15em;
		text-transform: uppercase;
		color: var(--muted-2);
	}
	.count {
		font-family: var(--font-mono);
		font-size: var(--type-11);
		line-height: 1;
		color: var(--muted-3);
		font-variant-numeric: tabular-nums;
	}
	.spacer {
		flex: 1;
	}
	button.ghost {
		font-size: var(--type-11-5);
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
	.state {
		font-size: var(--type-9-5);
		font-weight: 600;
		line-height: 1;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		padding: 4px 7px;
		border-radius: 5px;
		color: var(--accent-soft);
		background: var(--accent-chip-bg);
	}
	.state.unsaved {
		color: var(--warn-soft);
		background: var(--warn-bg);
	}
	button.ghost.quiet.confirming {
		color: var(--danger-soft);
		border-color: var(--danger-border);
	}
	.saved {
		margin: 0;
		font-size: var(--type-12);
		color: var(--accent-soft);
		word-break: break-all;
	}

	.saved code,
	.hint code {
		font-family: var(--font-mono);
		font-size: var(--type-11);
	}
	.hint {
		margin: 0;
		font-size: var(--type-12-5);
		line-height: 1.55;
		color: var(--muted-3);
		word-break: break-word;
	}
	.warn {
		margin: 0;
		padding: 9px 11px;
		border-radius: 8px;
		border: 1px solid var(--warn-border);
		background: var(--warn-bg);
		color: var(--warn-soft);
		font-size: var(--type-12);
		line-height: 1.55;
		text-wrap: pretty;
	}
	.recovery {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 9px;
		align-items: start;
		cursor: pointer;
	}
	.recovery input {
		margin: 2px 0 0;
		accent-color: var(--accent);
	}
	.recovery-title {
		display: block;
		font-size: var(--type-12);
		color: var(--text-soft);
	}
	.recovery-note {
		display: block;
		margin-top: 3px;
		font-size: var(--type-11-5);
		line-height: 1.5;
		color: var(--muted-3);
		text-wrap: pretty;
	}
	.log {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 10px;
		/* 180px at 100% — about six rows, in `em` so it stays about six rows. */
		max-height: 11.25em;
		overflow-y: auto;
	}
	.log li {
		display: grid;
		grid-template-columns: 4.25em 1fr;
		gap: 14px;
	}
	.side {
		font-size: var(--type-9-5);
		font-weight: 500;
		line-height: 1.6;
		letter-spacing: 0.12em;
		text-transform: uppercase;
	}
	.origin-microphone .side {
		color: var(--muted-3);
	}
	/* Dimmer than --room-soft: in the log the label is a marker, not a heading. */
	.origin-system .side {
		color: #6b8fd6;
	}
	.text {
		font-size: var(--type-12-5);
		line-height: 1.55;
		color: var(--muted);
		text-wrap: pretty;
	}
</style>
