<script lang="ts">
	import { untrack } from 'svelte';
	import { t } from './i18n';
	import { captionDirection } from './languages';
	import { loggedLanguage, options } from './stores';
	import { groupTranscript } from './transcript';
	import { laneLanguage, type Origin, type OutputMode, type TranscriptLine } from './types';
	import ToolButton from './ui/ToolButton.svelte';

	/** The transcript as it will be saved — chronological, grouped into readable paragraphs,
	 *  newest at the bottom — and the turns still being spoken under it in the reader. It
	 *  follows new text while the operator is at the bottom, and offers a way back when not. */
	let {
		mode,
		transcript,
		pending,
		reader,
		showOriginal
	}: {
		mode: OutputMode;
		transcript: TranscriptLine[];
		/** The turns still being spoken, oldest first: shown only in the reader. */
		pending: TranscriptLine[];
		/** The stage's transcript reader (issue #124): the log fills the stage. */
		reader: boolean;
		/** The original speech under each paragraph, exactly when it would be exported. */
		showOriginal: boolean;
	} = $props();

	// The log names the two sides of the room; the saved file names the devices
	// (`export.origin`). Both follow the interface language.
	const sideLabel = $derived<Record<Origin, string>>($t.transcript.side);

	const paragraphs = $derived(groupTranscript(transcript));

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
		void [paragraphs, pending, reader];
		if (!paragraphs.length && !pending.length) following = true;
		if (logEl && untrack(() => following) && !selecting()) logEl.scrollTop = logEl.scrollHeight;
	});

	/** Ctrl+A in the log selects the log, not the whole window around it. */
	function selectLog(event: KeyboardEvent) {
		if (!(event.ctrlKey || event.metaKey) || event.key.toLowerCase() !== 'a' || !logEl) return;
		event.preventDefault();
		document.getSelection()?.selectAllChildren(logEl);
	}
</script>

<!-- Read out with each turn still being spoken, rather than written into the text a
     selection would copy. -->
<span id={pendingNoteId} hidden>{$t.transcript.inProgress}</span>

<!-- The caption text carries the language it is in, as the overlay's does (review U12), so
     Narrator reads it with that language's voice and a right-to-left language lays out as
     one. Unknown — a subtitle engine detecting the language, a recovered line — is `lang=""`
     with the direction taken from the text. The original speech is in the spoken language,
     which this window does not know. -->
{#snippet caption(text: string, language: string | undefined, interim: boolean, original: string)}
	<span class="text" lang={language ?? ''} dir={captionDirection(language)}
		>{text}{#if interim}<span class="caret" aria-hidden="true"></span>{/if}{#if original}<span
				class="original"
				lang=""
				dir="auto">{original}</span
			>{/if}</span
	>
{/snippet}

{#if paragraphs.length || pending.length}
	<!-- Jump to latest floats over the log's foot rather than taking a row under it, so
	     scrolling up never pushes the rest of the panel down. -->
	<div class="log-frame" class:reader>
		<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions (Scrollable transcript needs keyboard scrolling, a focus target after Jump to latest, and Ctrl+A that selects only the log.) -->
		<div
			class="log-scroll"
			class:reader
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
						{@render caption(
							paragraph.text,
							loggedLanguage(paragraph),
							false,
							showOriginal && paragraph.lane === 0 ? paragraph.sourceText : ''
						)}
					</li>
				{/each}
				<!-- Keyed by track: a turn keeps its row while it streams, whatever id it has. -->
				{#each pending as line (`${line.origin}-${line.lane ?? 0}`)}
					<li class="origin-{line.origin} pending" aria-describedby={pendingNoteId}>
						<span class="side"
							>{sideLabel[line.origin]}{#if line.language}<span class="language"
									>{line.language}</span
								>{/if}</span
						>
						{@render caption(
							line.text,
							line.language ?? laneLanguage($options, line.lane ?? 0),
							true,
							showOriginal && (line.lane ?? 0) === 0 ? line.sourceText : ''
						)}
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
	</div>
{:else}
	<p class="hint">
		{mode === 'translate' ? $t.transcript.emptyTranslate : $t.transcript.emptySubtitles}
	</p>
{/if}

<style>
	.log-frame {
		position: relative;
		display: flex;
		flex-direction: column;
	}
	.log-frame.reader {
		flex: 1 1 auto;
		min-height: 0;
	}
	/* Over the log's bottom-right corner, raised off the text it covers. */
	.log-scroll ~ :global(.jump-latest) {
		position: absolute;
		right: var(--space-3);
		bottom: var(--space-2);
		max-width: calc(100% - 2 * var(--space-3));
		white-space: normal;
		background: var(--surface-2);
		box-shadow: var(--shadow-flyout);
	}
	.hint {
		margin: 0;
		font-size: var(--type-body);
		line-height: var(--leading-body);
		color: var(--text-muted);
		word-break: break-word;
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

	/* ---- Transcript reader ------------------------------------------------------ */

	/* Stacked, the page scrolls as a whole and the stage has no height of its own to fill,
	   so the log takes most of the window and keeps its own scrollbar. */
	.log-scroll.reader {
		flex: 1 1 auto;
		min-height: 11.25em;
		max-height: max(11.25em, 70vh);
	}
	@container window (min-width: 53.75em) {
		.log-scroll.reader {
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
