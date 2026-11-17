<script lang="ts">
	import { tick, untrack } from 'svelte';
	import { t } from './i18n';
	import { captionDirection } from './languages';
	import { loggedLanguage, options } from './stores';
	import { groupTranscript, markerTime, markerTitle, withMarkers } from './transcript';
	import {
		createFindIndex,
		keepCurrent,
		stepMatch,
		type FindMatch,
		type FindTarget
	} from './transcriptFind';
	import {
		laneLanguage,
		type Origin,
		type OutputMode,
		type TranscriptLine,
		type TranscriptMarker
	} from './types';
	import ToolButton from './ui/ToolButton.svelte';
	import TranscriptFind from './TranscriptFind.svelte';

	/** The transcript as it will be saved — chronological, grouped into readable paragraphs,
	 *  newest at the bottom — and the turns still being spoken under it in the reader. It
	 *  follows new text while the operator is at the bottom, and offers a way back when not. */
	let {
		mode,
		transcript,
		pending,
		reader,
		showOriginal,
		markers = [],
		findOpen = $bindable(false),
		findRequest = 0
	}: {
		mode: OutputMode;
		transcript: TranscriptLine[];
		/** The operator's markers, drawn between the paragraphs where they were added. */
		markers?: readonly TranscriptMarker[];
		/** The find bar is open over the log (reader only). */
		findOpen?: boolean;
		/** Changes each time the window asks for find (Ctrl+F), to focus its field again. */
		findRequest?: number;
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

	const paragraphs = $derived(groupTranscript(transcript, markers));
	// The paragraphs with the markers between them, in the order they were added.
	const items = $derived(withMarkers(paragraphs, markers));

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
		// Never while find is open: the view stays on the match being read, and only says
		// whether it is still at the bottom.
		void [items, pending, reader];
		if (!items.length && !pending.length) following = true;
		if (findOpen) untrack(noteScroll);
		else if (logEl && untrack(() => following) && !selecting())
			logEl.scrollTop = logEl.scrollHeight;
	});

	// ---- Find (reader only) --------------------------------------------------------------
	// Searched after a short pause in typing, and again when the log gains a line or a marker;
	// never per caption delta, since the turns still being spoken are not searched.
	const FIND_DEBOUNCE_MS = 150;
	/** The most matches painted at once. All are counted and can be stepped to; past this,
	 *  only the current one is highlighted, so a one-letter query in a day-long log stays
	 *  cheap. */
	const MAX_HIGHLIGHTS = 1000;
	const findIndex = createFindIndex();
	let query = $state('');
	let searched = $state('');
	let current = $state(-1);
	let matches = $state.raw<FindMatch[]>([]);
	let searchedOnce = $state(false);

	$effect(() => {
		const next = query;
		if (!findOpen) return;
		const timer = setTimeout(() => (searched = next), FIND_DEBOUNCE_MS);
		return () => clearTimeout(timer);
	});

	const sectionLabel = $derived($t.marker.section);
	const targets = $derived.by<FindTarget[]>(() => {
		if (!findOpen) return [];
		return items.flatMap((item): FindTarget[] => {
			if (item.kind === 'marker')
				return [
					{
						key: `m${item.marker.id}`,
						text: markerTitle(item.marker, item.number, sectionLabel)
					}
				];
			const p = item.paragraph;
			const original = showOriginal && p.lane === 0 && p.sourceText;
			return [
				{ key: `p${p.id}`, text: p.text },
				...(original ? [{ key: `o${p.id}`, text: p.sourceText }] : [])
			];
		});
	});

	$effect(() => {
		const list = targets;
		const q = searched;
		if (!findOpen) return;
		untrack(() => {
			const previous = matches[current];
			const order = new Map(list.map((target, i) => [target.key, i]));
			const next = findIndex.search(list, q);
			current = keepCurrent(previous, next, (key) => order.get(key) ?? -1);
			matches = next;
			searchedOnce = q.trim() !== '';
		});
	});

	function step(direction: 1 | -1) {
		current = stepMatch(current, matches.length, direction);
		scrolledTo = '';
	}

	/** Close find: no highlights, following as before, and focus back where it came from. */
	function closeFind(returnTo: HTMLElement | null) {
		findOpen = false;
		query = '';
		searched = '';
		matches = [];
		current = -1;
		searchedOnce = false;
		clearHighlights();
		noteScroll();
		if (returnTo?.isConnected) returnTo.focus();
		else logEl?.focus({ preventScroll: true });
	}

	const highlights = () =>
		typeof CSS !== 'undefined' && 'highlights' in CSS && typeof Highlight === 'function'
			? CSS.highlights
			: null;

	function clearHighlights() {
		highlights()?.delete('transcript-find');
		highlights()?.delete('transcript-find-current');
	}
	$effect(() => clearHighlights);

	/** Where each piece of searchable text is drawn: the text node inside its element. */
	function textNodes(): Map<string, Text> {
		const nodes = new Map<string, Text>();
		for (const element of logEl?.querySelectorAll<HTMLElement>('[data-find]') ?? []) {
			const node = Array.from(element.childNodes).find(
				(child): child is Text => child.nodeType === Node.TEXT_NODE
			);
			if (node && element.dataset.find) nodes.set(element.dataset.find, node);
		}
		return nodes;
	}

	function rangeOf(nodes: Map<string, Text>, match: FindMatch): Range | null {
		const node = nodes.get(match.key);
		if (!node || match.end > node.length) return null;
		const range = document.createRange();
		range.setStart(node, match.start);
		range.setEnd(node, match.end);
		return range;
	}

	// The match last scrolled to, so a new line does not pull the view back to it after the
	// operator has scrolled away; stepping or a new query scrolls again.
	let scrolledTo = '';

	$effect(() => {
		const list = matches;
		const at = current;
		void items;
		if (!findOpen) return;
		void tick().then(() => {
			if (!findOpen || !logEl) return;
			const nodes = textNodes();
			const registry = highlights();
			const currentRange = at >= 0 ? rangeOf(nodes, list[at]) : null;
			if (registry) {
				const all = list
					.slice(0, MAX_HIGHLIGHTS)
					.map((match) => rangeOf(nodes, match))
					.filter((range): range is Range => range !== null);
				registry.set('transcript-find', new Highlight(...all));
				if (currentRange) registry.set('transcript-find-current', new Highlight(currentRange));
				else registry.delete('transcript-find-current');
			}
			const match = list[at];
			const key = match ? `${match.key}:${match.start}` : '';
			if (currentRange && key !== scrolledTo) {
				scrolledTo = key;
				const box = currentRange.getBoundingClientRect();
				const frame = logEl.getBoundingClientRect();
				if (box.top < frame.top || box.bottom > frame.bottom)
					logEl.scrollTop += box.top - frame.top - (frame.height - box.height) / 2;
				noteScroll();
			}
		});
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
{#snippet caption(
	text: string,
	language: string | undefined,
	interim: boolean,
	original: string,
	findKey: string | undefined
)}
	<span
		class="text"
		lang={language ?? ''}
		dir={captionDirection(language)}
		data-find={findKey ? `p${findKey}` : undefined}
		>{text}{#if interim}<span class="caret" aria-hidden="true"></span>{/if}{#if original}<span
				class="original"
				lang=""
				dir="auto"
				data-find={findKey ? `o${findKey}` : undefined}>{original}</span
			>{/if}</span
	>
{/snippet}

{#if reader && findOpen}
	<TranscriptFind
		bind:query
		total={searchedOnce ? matches.length : null}
		{current}
		focusRequest={findRequest}
		onStep={step}
		onClose={closeFind}
	/>
{/if}

{#if items.length || pending.length}
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
				{#each items as item (item.kind === 'marker' ? `m${item.marker.id}` : item.paragraph.id)}
					{#if item.kind === 'marker'}
						<!-- A marker is a heading inside the log, so a screen reader can move from one
						     talk to the next by heading. -->
						<li class="marker">
							<span class="marker-time">{markerTime(item.marker.atMs)}</span>
							<h3 class="marker-title" data-find="m{item.marker.id}">
								{markerTitle(item.marker, item.number, sectionLabel)}
							</h3>
						</li>
					{:else}
						{@const paragraph = item.paragraph}
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
								showOriginal && paragraph.lane === 0 ? paragraph.sourceText : '',
								String(paragraph.id)
							)}
						</li>
					{/if}
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
							showOriginal && (line.lane ?? 0) === 0 ? line.sourceText : '',
							undefined
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
	/* A marker divides the log: a rule above, its time where a speaker's label would be, and
	   its title in the accent, so a talk's start is found by eye in a long log. */
	.log li.marker {
		align-items: baseline;
		padding-top: var(--space-2);
		border-top: 1px solid var(--line-strong);
	}
	.marker-time {
		padding-top: 2px;
		font-family: var(--font-mono);
		font-size: var(--type-caption);
		font-weight: 500;
		line-height: var(--leading-body);
		color: var(--text-muted);
		font-variant-numeric: tabular-nums;
	}
	.marker-title {
		margin: 0;
		font-size: var(--type-body);
		font-weight: 600;
		line-height: var(--leading-body);
		color: var(--accent-soft);
		overflow-wrap: anywhere;
	}

	/* Find: every match marked, the current one stronger. Drawn by the CSS Custom Highlight
	   API, which leaves the text, its selection and what Copy takes exactly as they were. */
	:global(::highlight(transcript-find)) {
		background-color: var(--warn-border);
		color: var(--text-body);
	}
	:global(::highlight(transcript-find-current)) {
		background-color: var(--warn);
		color: var(--surface-0);
	}
	@media (forced-colors: active) {
		:global(::highlight(transcript-find)) {
			background-color: Mark;
			color: MarkText;
		}
		:global(::highlight(transcript-find-current)) {
			background-color: Highlight;
			color: HighlightText;
		}
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
