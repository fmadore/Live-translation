<script lang="ts">
	import { untrack } from 'svelte';
	import { isTauri } from './tauri';
	import { recovery } from './recovery';
	import { t } from './i18n';
	import { asStatus, type AppError } from './errors';
	import {
		clearTranscript as clearTranscriptStore,
		exportOriginal,
		inFlightLines,
		recoveryEnabled,
		savedPath,
		transcriptDirty
	} from './stores';
	import { TRANSCRIPT_WARN_LINES } from './document';
	import { exportFormat, isTimedFormat } from './exportFormat';
	import { exportLabels, saveTranscriptDocument } from './saveDocument';
	import {
		formatTranscript,
		groupTranscript,
		hasOriginalSpeech,
		hasTranscriptTiming,
		recentLines,
		type TranscriptFormat
	} from './transcript';
	import type { Origin, OutputMode, TranscriptLine } from './types';
	import ToolButton from './ui/ToolButton.svelte';
	import Preference from './ui/Preference.svelte';

	interface Props {
		mode: OutputMode;
		transcript: TranscriptLine[];
		onError: (message: string | AppError) => void;
		/** A session is running, so turns still being spoken belong in what Copy takes. */
		live?: boolean;
		/** The stage's transcript reader (issue #124): the log fills the stage instead of six
		 *  rows under the live turns, shows the turns still being spoken at its foot, and the
		 *  original speech under each paragraph when it is set to be exported. */
		reader?: boolean;
	}

	let { mode, transcript, onError, live = false, reader = false }: Props = $props();
	// A browser preview has no filesystem to spool to, and inviting the operator to switch on a
	// feature whose only outcome would be an IPC error is the mistake issue #29 fixed elsewhere.
	const desktop = isTauri();
	let saving = $state(false);
	// The export format is a stored preference (`exportFormat.ts`), not this component's
	// state: the page mounts the monitor once while a session runs and again after it, and
	// each mount used to start over from Markdown.
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

	// The log names the two sides of the room; the saved file names the devices
	// (`export.origin`). Both follow the interface language.
	const sideLabel = $derived<Record<Origin, string>>($t.transcript.side);

	// Show the log exactly as it will be saved: chronological, grouped into readable
	// paragraphs, newest at the bottom.
	const paragraphs = $derived(groupTranscript(transcript));

	// Turns not yet final. Only a running session has any: after Stop they are committed.
	const pending = $derived(live ? $inFlightLines : []);
	const shownPending = $derived(reader ? pending : []);
	// The reader shows what Save and Copy will write, so the original speech appears under
	// each paragraph exactly when it would be exported.
	const showOriginal = $derived(reader && bilingual && $exportOriginal);

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

	// Nothing is ever dropped (issue #25), but a log this long is a session worth putting on
	// disk before something else decides for the operator.
	const veryLong = $derived(transcript.length >= TRANSCRIPT_WARN_LINES);

	let logEl = $state<HTMLDivElement | null>(null);
	let following = $state(true);
	const idBase = $props.id();
	const logId = `${idBase}-transcript-log`;
	const pendingNoteId = `${idBase}-in-progress`;

	function noteScroll() {
		if (logEl) following = logEl.scrollHeight - logEl.clientHeight - logEl.scrollTop <= 24;
	}

	function jumpToLatest() {
		following = true;
		if (logEl) {
			logEl.scrollTop = logEl.scrollHeight;
			logEl.focus({ preventScroll: true });
		}
	}

	/** Whether the operator has text selected in the log, which a scroll would drag along. */
	function selecting(): boolean {
		const selection = document.getSelection();
		return !!logEl && !!selection && !selection.isCollapsed && logEl.contains(selection.anchorNode);
	}

	$effect(() => {
		// Follow incoming text only while the operator is already at the bottom, and not
		// while they are selecting there. A change of view resizes the log, so it pins again.
		void [paragraphs, shownPending, reader];
		if (!paragraphs.length && !shownPending.length) following = true;
		if (logEl && untrack(() => following) && !selecting()) logEl.scrollTop = logEl.scrollHeight;
	});

	/** Ctrl+A in the log selects the log, not the whole window around it. */
	function selectLog(event: KeyboardEvent) {
		if (!(event.ctrlKey || event.metaKey) || event.key.toLowerCase() !== 'a' || !logEl) return;
		event.preventDefault();
		document.getSelection()?.selectAllChildren(logEl);
	}

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

	function toggleRecovery(enabled: boolean) {
		if (!desktop) return;
		recoveryEnabled.set(enabled);
		// Switching it off has to remove what it already wrote, or "disabled" would still
		// leave captions sitting on disk.
		if (!enabled) void recovery.clear().catch((error) => onError(asStatus(error)));
	}
</script>

<section class="monitor" class:reader>
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
		</div>
		<select aria-label={$t.transcript.format} bind:value={$exportFormat} disabled={saving}>
			<option value="markdown">Markdown (.md)</option>
			<option value="text">{$t.transcript.plainText} (.txt)</option>
			<option value="vtt">WebVTT (.vtt)</option>
			<option value="srt">SubRip (.srt)</option>
		</select>
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
	{#if copyNotice}<p class="saved" aria-hidden="true">{copyNotice}</p>{/if}
	<!-- Read out with each turn still being spoken, rather than written into the text a
	     selection would copy. -->
	<span id={pendingNoteId} hidden>{$t.transcript.inProgress}</span>
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

	{#if paragraphs.length || shownPending.length}
		<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions (Scrollable transcript needs keyboard scrolling, a focus target after Jump to latest, and Ctrl+A that selects only the log.) -->
		<div
			class="log-scroll"
			id={logId}
			role="region"
			aria-label={$t.transcript.heading}
			tabindex="0"
			onscroll={noteScroll}
			onkeydown={selectLog}
			bind:this={logEl}
		>
			<ul class="log">
				{#each paragraphs as paragraph (paragraph.id)}
					<li class="origin-{paragraph.origin}">
						<span class="side"
							>{sideLabel[paragraph.origin]}{#if paragraph.language}<span class="language"
									>{paragraph.language}</span
								>{/if}</span
						>
						<span class="text"
							>{paragraph.text}{#if showOriginal && paragraph.lane === 0 && paragraph.sourceText}<span
									class="original">{paragraph.sourceText}</span
								>{/if}</span
						>
					</li>
				{/each}
				<!-- Keyed by track: a turn keeps its row while it streams, whatever id it has. -->
				{#each shownPending as line (`${line.origin}-${line.lane ?? 0}`)}
					<li class="origin-{line.origin} pending" aria-describedby={pendingNoteId}>
						<span class="side"
							>{sideLabel[line.origin]}{#if line.language}<span class="language"
									>{line.language}</span
								>{/if}</span
						>
						<span class="text"
							>{line.text}<span class="caret" aria-hidden="true"
							></span>{#if showOriginal && (line.lane ?? 0) === 0 && line.sourceText}<span
									class="original">{line.sourceText}</span
								>{/if}</span
						>
					</li>
				{/each}
			</ul>
		</div>
		{#if !following}
			<ToolButton
				variant="ghost"
				size="sm"
				class="jump-latest"
				aria-controls={logId}
				onclick={jumpToLatest}
			>
				{$t.transcript.jumpToLatest}
			</ToolButton>
		{/if}
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
	.monitor :global(.jump-latest) {
		align-self: flex-end;
		white-space: normal;
	}
	.monitor {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		margin-top: var(--space-5);
		padding-top: var(--space-4);
		border-top: 1px solid var(--line);
		/* The stage is a scrolling flex column; keep the block at its natural height there. */
		flex: 0 0 auto;
	}
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
	.state.unsaved {
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
	.recovery {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: var(--space-2);
		align-items: start;
		cursor: pointer;
	}
	.recovery input {
		margin: 2px 0 0;
		accent-color: var(--accent);
	}
	.recovery-title {
		display: block;
		font-size: var(--type-small);
		color: var(--text-secondary);
	}
	.recovery-note {
		display: block;
		margin-top: var(--space-1);
		font-size: var(--type-small);
		line-height: var(--leading-body);
		color: var(--text-muted);
		text-wrap: pretty;
	}
	.log {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.log-scroll {
		/* 180px at 100% — about six rows, in `em` so it stays about six rows. */
		max-height: 11.25em;
		overflow-y: auto;
	}
	.log li {
		display: grid;
		grid-template-columns: 4.25em 1fr;
		gap: var(--space-3);
	}
	/* Two pixels down, so the small-capital label sits on the baseline of the first line of
	   the larger text beside it rather than above it. */
	.side {
		padding-top: 2px;
		font-size: var(--type-caption);
		font-weight: 500;
		line-height: var(--leading-body);
		letter-spacing: var(--tracking-caps);
		text-transform: uppercase;
	}
	/* The same identity as the live turns above: blue is the room, and only the room. The
	   remote side is the default speaker, so its marker stays neutral. */
	.origin-microphone .side {
		color: var(--room-soft);
	}
	/* With two caption languages the paragraphs alternate, so each names its language under
	   the speaker, as a code: the column is too narrow for a name. */
	.language {
		display: block;
		font-family: var(--font-mono);
		color: var(--text-muted);
	}
	.origin-system .side {
		color: var(--text-muted);
	}
	.text {
		font-size: var(--type-body);
		line-height: var(--leading-body);
		color: var(--text-muted);
		text-wrap: pretty;
	}

	.copy {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	/* ---- Transcript reader ------------------------------------------------------ */

	/* The stage's whole height, its own scrollbar for the log, and the rest of the block at
	   its natural height around it. No rule above: the tabs already divide it from the bar. */
	.monitor.reader {
		flex: 1 1 auto;
		min-height: 0;
		margin-top: var(--space-4);
		padding-top: 0;
		border-top: 0;
	}
	/* Stacked, the page scrolls as a whole and the stage has no height of its own to fill,
	   so the log takes most of the window and keeps its own scrollbar. */
	.reader .log-scroll {
		flex: 1 1 auto;
		min-height: 11.25em;
		max-height: max(11.25em, 70vh);
	}
	@container window (min-width: 53.75em) {
		.reader .log-scroll {
			max-height: none;
		}
	}
	/* The text being read, not a record of what has gone by: body colour, not muted. */
	.reader .text {
		color: var(--text-body);
	}
	.reader .log {
		gap: var(--space-3);
	}
	.original {
		display: block;
		margin-top: var(--space-1);
		font-size: var(--type-small);
		color: var(--text-muted);
	}
	.caret {
		display: inline-block;
		width: 3px;
		height: 0.9em;
		background: var(--accent);
		margin-left: var(--space-2);
		vertical-align: -2px;
		animation: blink 1.1s steps(1) infinite;
	}
	@media (prefers-reduced-motion: reduce) {
		.caret {
			animation: none;
		}
	}
</style>
