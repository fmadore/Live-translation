<script lang="ts">
	import { isTauri } from './tauri';
	import { recovery, recoveryFailure } from './recovery';
	import { historyEnabled, historyError } from './history';
	import { t } from './i18n';
	import { asStatus, describeError, type AppError } from './errors';
	import {
		clearTranscript as clearTranscriptStore,
		exportOriginal,
		recoveryEnabled,
		savedPath,
		transcriptDirty
	} from './stores';
	import { TRANSCRIPT_WARN_LINES } from './document';
	import { exportFormat, isTimedFormat } from './exportFormat';
	import { exportLabels, saveTranscriptDocument } from './saveDocument';
	import {
		formatTranscript,
		hasOriginalSpeech,
		hasTranscriptTiming,
		recentLines,
		type TranscriptFormat
	} from './transcript';
	import type { TranscriptLine } from './types';
	import ToolButton from './ui/ToolButton.svelte';
	import Preference from './ui/Preference.svelte';
	import ExportFormatSelect from './ExportFormatSelect.svelte';

	/** The transcript's toolbar and the notes under it: its state, Copy, Save, Clear, and what
	 *  each of them last did. The log itself is `TranscriptLog`. */
	let {
		transcript,
		pending,
		onError
	}: {
		transcript: TranscriptLine[];
		/** The turns still being spoken, oldest first, which Copy takes with the log. */
		pending: TranscriptLine[];
		onError: (message: string | AppError) => void;
	} = $props();

	const desktop = isTauri();
	let saving = $state(false);
	// The export format is a stored preference (`exportFormat.ts`), not this component's
	// state, so the quit prompt's Save and the history view use the same one.
	const timedExportAvailable = $derived(hasTranscriptTiming(transcript));
	// Offered only when there is something to include: subtitles have no separate original.
	const bilingual = $derived(hasOriginalSpeech(transcript));
	const missingTiming = $derived(isTimedFormat($exportFormat) && !timedExportAvailable);
	// Clear throws away unsaved text, so when there is any it asks once rather than acting on
	// the first click. New captions withdraw the question — the log the operator was about to
	// throw away is no longer the log in front of them — so it is derived from the transcript,
	// and only the click that asks sets it.
	let confirmingClear = $derived.by(() => {
		void transcript.length;
		return false;
	});

	// Copy takes the end of the conversation for an assistant or a chat (issue #124). A time
	// window needs every line placed on the timeline; a recovered log without timing is
	// copied whole.
	const COPY_WINDOWS = [5, 10, 30, 0] as const;
	let copyMinutes = $state<(typeof COPY_WINDOWS)[number]>(10);
	const copyTimed = $derived(hasTranscriptTiming(transcript));
	const copyWindow = $derived(copyTimed ? copyMinutes : 0);
	let copying = $state(false);
	let copyNotice = $state('');
	let noticeTimer: ReturnType<typeof setTimeout> | undefined;
	$effect(() => () => clearTimeout(noticeTimer));

	// What a screen reader hears after a save. Empty while there is nothing new to report, so
	// the region stays silent between saves.
	const saveAnnouncement = $derived(
		$savedPath && !$transcriptDirty ? $t.transcript.savedAnnouncement($savedPath) : ''
	);

	// Background saving that keeps failing says so here for as long as it lasts (review U3):
	// the status line words a spool failure once, and the next status overwrites it, and a
	// history failure was shown only in Settings. Each is the failure as last caught, so the
	// tooltip carries its detail.
	const spoolFailing = $derived($recoveryEnabled && !!$recoveryFailure);
	const historyFailing = $derived($historyEnabled && !!$historyError);

	// Nothing is ever dropped (issue #25), but a log this long is a session worth putting on
	// disk before something else decides for the operator.
	const veryLong = $derived(transcript.length >= TRANSCRIPT_WARN_LINES);

	async function copy() {
		if (copying) return;
		copying = true;
		clearTimeout(noticeTimer);
		copyNotice = '';
		try {
			// Newest first, like the log: the turns still being spoken are the newest text.
			const lines = [...pending].reverse().concat(transcript);
			const excerpt = copyWindow ? recentLines(lines, copyWindow * 60_000) : lines;
			await navigator.clipboard.writeText(
				formatTranscript(excerpt, 'text', new Date(), exportLabels(), {
					original: $exportOriginal
				})
			);
			copyNotice = $t.transcript.copied(copyWindow);
			noticeTimer = setTimeout(() => (copyNotice = ''), 4000);
		} catch (error) {
			onError(asStatus(error));
		} finally {
			copying = false;
		}
	}

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
		if (desktop) void recovery.clear().catch(() => {});
	}
</script>

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
	{#if spoolFailing}<span class="state failing" title={describeError($recoveryFailure, $t)}
			>{$t.transcript.recoveryFailing}</span
		>{/if}
	{#if historyFailing}<span class="state failing" title={describeError($historyError, $t)}
			>{$t.transcript.historyFailing}</span
		>{/if}
	<div class="spacer"></div>
	<div class="copy">
		<!-- Shows the window Copy will actually use: the whole log when it has no timing. -->
		<select
			aria-label={$t.transcript.copyRange}
			value={copyWindow}
			disabled={copying || !copyTimed}
			onchange={(e) => (copyMinutes = Number(e.currentTarget.value) as typeof copyMinutes)}
		>
			{#each COPY_WINDOWS as minutes (minutes)}
				<option value={minutes}>{$t.transcript.copyWindow(minutes)}</option>
			{/each}
		</select>
		<ToolButton
			variant="ghost"
			size="sm"
			disabled={(!transcript.length && !pending.length) || copying}
			aria-busy={copying}
			onclick={copy}
		>
			{$t.transcript.copy}
		</ToolButton>
		<!-- Floats under Copy for its few seconds rather than taking a row above the log, which
		     pushed the log down and pulled it back up. -->
		{#if copyNotice}<p class="copied" aria-hidden="true">{copyNotice}</p>{/if}
	</div>
	<ExportFormatSelect compact disabled={saving} />
	{#if bilingual}<Preference
			label={$t.transcript.includeOriginal}
			checked={$exportOriginal}
			disabled={saving}
			onchange={(value) => exportOriginal.set(value)}
		/>{/if}
	<ToolButton
		variant="ghost"
		size="sm"
		disabled={!desktop || !transcript.length || saving || missingTiming}
		aria-busy={saving}
		onclick={() => save($exportFormat)}
	>
		{$t.transcript.saveAs}
	</ToolButton>
	<ToolButton
		variant={confirmingClear ? 'danger' : 'ghost'}
		size="sm"
		disabled={!transcript.length || saving}
		onclick={clear}
	>
		{confirmingClear ? $t.transcript.confirmClear : $t.transcript.clear}
	</ToolButton>
</div>
{#if missingTiming && transcript.length}<p class="hint" role="status">
		{$t.transcript.noTiming}
	</p>{/if}

<!-- The announcement lives in its own always-present region — a live region created at the
     same moment as its text is routinely missed, and this one is out of flow, so it costs
     the layout nothing between saves. The visible lines below repeat it for the eye. -->
<p class="sr-only" role="status">{saveAnnouncement}</p>
<!-- The same arrangement for Copy, which a screen reader otherwise does not hear at all. -->
<p class="sr-only" role="status">{copyNotice}</p>
<!-- History saving that starts failing, said once. A failing recovery copy needs no region of
     its own: its first failure reaches the window's status line, which is one already. -->
<p class="sr-only" role="status">{historyFailing ? $t.transcript.historyFailing : ''}</p>
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

<style>
	/* Wraps rather than squeezing its buttons below their labels at a large text size. */
	.head {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-3);
	}
	.count {
		font-family: var(--font-mono);
		font-size: var(--type-caption);
		line-height: 1;
		color: var(--text-muted);
		font-variant-numeric: tabular-nums;
	}
	.spacer {
		flex: 1;
	}
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
	.state {
		font-size: var(--type-caption);
		font-weight: 600;
		line-height: 1;
		letter-spacing: var(--tracking-caps);
		text-transform: uppercase;
		padding: var(--space-1) var(--space-2);
		border-radius: var(--radius-control);
		color: var(--accent-soft);
		background: var(--accent-chip);
	}
	.state.unsaved,
	.state.failing {
		color: var(--warn-soft);
		background: var(--warn-chip);
	}
	.saved {
		margin: 0;
		font-size: var(--type-small);
		color: var(--accent-soft);
		word-break: break-all;
	}

	.saved code,
	.hint code {
		font-family: var(--font-mono);
		font-size: var(--type-caption);
	}
	.hint {
		margin: 0;
		font-size: var(--type-body);
		line-height: var(--leading-body);
		color: var(--text-muted);
		word-break: break-word;
	}
	.warn {
		margin: 0;
		padding: var(--space-2) var(--space-3);
		border-radius: var(--radius-control);
		border: 1px solid var(--warn-border);
		background: var(--warn-wash);
		color: var(--warn-soft);
		font-size: var(--type-small);
		line-height: var(--leading-body);
		text-wrap: pretty;
	}

	.copy {
		position: relative;
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}
	.copied {
		position: absolute;
		top: calc(100% + var(--space-1));
		right: 0;
		z-index: 1;
		margin: 0;
		padding: var(--space-1) var(--space-2);
		white-space: nowrap;
		font-size: var(--type-small);
		color: var(--accent-soft);
		background: var(--surface-2);
		border: 1px solid var(--line-strong);
		border-radius: var(--radius-control);
		box-shadow: var(--shadow-flyout);
		pointer-events: none;
	}
</style>
