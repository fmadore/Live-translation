<script lang="ts">
	import { TARGET_LANGUAGES, captionDirection, languageName } from './languages';
	import Select from './ui/Select.svelte';
	import Field from './ui/Field.svelte';
	import DateField from './ui/DateField.svelte';
	import ToolButton from './ui/ToolButton.svelte';
	import ConfirmButton from './ui/ConfirmButton.svelte';
	import Preference from './ui/Preference.svelte';
	import ExportFormatSelect from './ExportFormatSelect.svelte';
	import { captionLanguagesOf, matchesSession } from './historySearch';
	import { onMount } from 'svelte';
	import { SvelteSet } from 'svelte/reactivity';
	import { api, isTauri } from './tauri';
	import { asStatus, type AppError } from './errors';
	import { describeStatus, type WordedMessage } from './sessionStatus';
	import { t, locale, localeTag, formatDateTime } from './i18n';
	import { formatElapsed } from './sessionClock.svelte';
	import {
		createHistoryCache,
		historyEnabled,
		historyError,
		historyRevision,
		sessionHistory,
		type ExportFile,
		type HistoryEntry,
		type SavedSession
	} from './history';
	import { exportOriginal, statusMessage } from './stores';
	import RecoveryPreference from './RecoveryPreference.svelte';
	// The same remembered choice as the transcript toolbar's, like the original-speech option.
	import { exportFormat as format, isTimedFormat } from './exportFormat';
	import {
		formatTranscript,
		transcriptFilename,
		hasOriginalSpeech,
		hasTranscriptTiming,
		markerTime,
		markerTitle,
		type TranscriptFormat
	} from './transcript';
	import type { TranscriptLine, TranscriptMarker } from './types';
	const desktop = isTauri();
	// Raw rather than deeply reactive: the list is only ever replaced, and an unchanged session
	// has to stay the very object the cache holds for the search's own cache to find it. A deep
	// proxy would hand out a new wrapper with every listing.
	let sessions = $state.raw<HistoryEntry[]>([]);
	const cache = createHistoryCache();
	let selectedId = $state('');
	let busy = $state(false);
	// Kept as caught and worded where it is shown, so the core's failures read in the interface
	// language rather than as "[object Object]". A failed listing and a failed action on the
	// selected session are told apart: each has its own headline, and a listing that succeeds
	// clears only its own failure, not the one an action is still reporting.
	let error = $state.raw<Caught>('');
	let actionError = $state.raw<Caught>('');
	// Worded where it is shown, like the failures, so it follows a change of interface language.
	let notice = $state.raw<WordedMessage | ''>('');
	type Caught = string | AppError | WordedMessage;
	/** A failure as caught: the core's `{ id, detail? }` or a message worded from the catalogue
	 *  stays structured until it is shown; anything else becomes its text. */
	function caught(e: unknown): Caught {
		return typeof e === 'function' ? (e as WordedMessage) : asStatus(e);
	}
	let confirmDelete = $state('');
	const selected = $derived(sessions.find((s) => s.id === selectedId)?.session);
	const timed = $derived(selected ? hasTranscriptTiming(selected.lines) : false);
	let query = $state('');
	let from = $state('');
	let to = $state('');
	let language = $state('');
	let title = $state('');
	const filtered = $derived(
		sessions.filter((entry) =>
			entry.session
				? matchesSession(entry.session, { query, from, to, language })
				: !query && !from && !to && !language
		)
	);
	// The sessions ticked for a batch action, apart from the one open in the reader. Only the
	// ticked sessions the filters show are acted on and counted, so a search narrows what
	// select-all takes and nothing hidden is ever deleted with the rest.
	const checked = new SvelteSet<string>();
	const chosen = $derived(filtered.filter((entry) => checked.has(entry.id)));
	const allShown = $derived(filtered.length > 0 && chosen.length === filtered.length);
	const chosenIds = $derived(chosen.map((entry) => entry.id).join(' '));
	// Armed for the sessions chosen when it was asked: whatever changes them (a tick, select-all,
	// a filter showing or hiding ticked sessions, a deletion) withdraws the question, so the
	// confirmation never covers sessions it did not count.
	let confirmBatch = $derived.by(() => {
		void chosenIds;
		return false;
	});
	// A batch's failures have their own headline, which counts them; `actionError`'s says
	// "this session".
	let batchError = $state.raw<{ failed: number; total: number; error: Caught } | null>(null);
	function toggleAll() {
		// Read once: `allShown` follows the set, so it would change after the first row.
		const untick = allShown;
		for (const entry of filtered) {
			if (untick) checked.delete(entry.id);
			else checked.add(entry.id);
		}
	}
	function toggle(id: string, on: boolean) {
		if (on) checked.add(id);
		else checked.delete(id);
	}
	/** The name a row's checkbox reads out: the session's title or start, as its button shows. */
	function entryName(entry: HistoryEntry) {
		if (!entry.session) return `${$t.history.unreadable} ${entry.id}`;
		return entry.session.title || formatDateTime(entry.session.startedAt, $localeTag);
	}
	function startBatch() {
		busy = true;
		confirmBatch = false;
		actionError = '';
		batchError = null;
		notice = '';
	}
	async function removeChosen() {
		const ids = chosen.map((entry) => entry.id);
		if (!ids.length || busy) return;
		startBatch();
		try {
			const outcome = await sessionHistory.deleteMany(ids);
			for (const id of outcome.deleted) checked.delete(id);
			if (outcome.deleted.includes(selectedId)) selectedId = '';
			const done = outcome.deleted.length;
			if (done) notice = (m) => m.history.deletedMany(done);
			if (outcome.failed.length)
				batchError = {
					failed: outcome.failed.length,
					total: ids.length,
					error: outcome.failed[0].error
				};
			await refresh();
		} catch (e) {
			batchError = { failed: ids.length, total: ids.length, error: caught(e) };
		} finally {
			confirmBatch = false;
			busy = false;
		}
	}
	async function exportChosen() {
		if (!chosen.length || busy) return;
		const wanted = $format;
		const files: ExportFile[] = [];
		let skipped = 0;
		for (const entry of chosen) {
			const session = entry.session;
			// Subtitles need each line's timing, as the single export's button says.
			if (!session || (isTimedFormat(wanted) && !hasTranscriptTiming(session.lines))) {
				skipped++;
				continue;
			}
			files.push({
				filename: transcriptFilename(new Date(session.startedAt), wanted),
				content: content(session, wanted)
			});
		}
		const total = chosen.length;
		startBatch();
		try {
			const outcome = files.length ? await api.exportTranscripts(files) : null;
			if (files.length && !outcome) return; // The folder dialog was cancelled.
			const written = outcome?.written.length ?? 0;
			const folder = outcome?.directory ?? '';
			notice = (m) =>
				[
					written ? m.history.savedMany(written, folder) : '',
					skipped ? m.history.skipped(skipped) : ''
				]
					.filter(Boolean)
					.join(' ');
			if (outcome?.failed.length)
				batchError = { failed: outcome.failed.length, total, error: outcome.failed[0].error };
		} catch (e) {
			batchError = { failed: files.length, total, error: caught(e) };
		} finally {
			busy = false;
		}
	}
	async function rename() {
		if (!selected || busy) return;
		busy = true;
		actionError = '';
		try {
			await sessionHistory.rename(selected, title);
			cache.forget(selected.id);
			await refresh();
			notice = (m) => m.usability.titleSaved;
		} catch (e) {
			actionError = caught(e);
		} finally {
			busy = false;
		}
	}
	let request = 0;

	async function refresh() {
		if (!desktop) return;
		const current = ++request;
		try {
			await sessionHistory.flush();
			const listing = await api.listHistory(cache.known());
			if (current !== request) return;
			sessions = cache.apply(listing);
			error = '';
		} catch (e) {
			if (current === request) error = caught(e);
		}
	}
	// A recording session writes history after every finalized line, and each write bumps the
	// revision. Re-listing every stored session for each one made an open History tab reread
	// the whole folder several times a minute, so the list catches up at most every few
	// seconds, and then reads only the sessions that changed. Rename, delete and the Refresh
	// button still refresh at once.
	const REFRESH_INTERVAL_MS = 5000;
	let lastRefresh = 0;
	let refreshTimer: ReturnType<typeof setTimeout> | undefined;
	onMount(() => () => {
		request++;
		clearTimeout(refreshTimer);
	});
	$effect(() => {
		void $historyRevision;
		if (refreshTimer !== undefined) return;
		const wait = lastRefresh + REFRESH_INTERVAL_MS - Date.now();
		const run = () => {
			refreshTimer = undefined;
			lastRefresh = Date.now();
			void refresh();
		};
		if (wait <= 0) run();
		else refreshTimer = setTimeout(run, wait);
	});

	/** The session's caption languages, both of them when it had two: `FR + DE`. */
	function captionLanguages(session: SavedSession) {
		const codes = captionLanguagesOf(session);
		return codes.length
			? codes.map((code) => code.toUpperCase()).join(' + ')
			: $t.history.sameLanguage;
	}

	function content(session: SavedSession, format: TranscriptFormat) {
		return formatTranscript(
			session.lines,
			format,
			new Date(session.startedAt),
			{
				title: $t.export.title,
				origin: $t.export.origin,
				original: $t.export.original,
				language: (code) => languageName(code, $locale),
				tag: $localeTag,
				section: $t.marker.section
			},
			{ original: $exportOriginal, markers: session.markers ?? [] }
		);
	}

	/** The session's lines, oldest first, with its markers among them where they were added
	 *  (lines and markers share one counter), each marker with its number. */
	function readerItems(session: SavedSession) {
		const markers = [...(session.markers ?? [])].sort((a, b) => a.id - b.id);
		const items: (
			| { kind: 'line'; line: TranscriptLine }
			| { kind: 'marker'; marker: TranscriptMarker; number: number }
		)[] = [];
		let next = 0;
		for (const line of [...session.lines].reverse()) {
			while (next < markers.length && markers[next].id < line.id) {
				items.push({ kind: 'marker', marker: markers[next], number: next + 1 });
				next++;
			}
			items.push({ kind: 'line', line });
		}
		for (; next < markers.length; next++)
			items.push({ kind: 'marker', marker: markers[next], number: next + 1 });
		return items;
	}
	async function action(kind: 'copy' | 'export') {
		if (!selected || busy) return;
		busy = true;
		actionError = '';
		notice = '';
		try {
			if (kind === 'copy') {
				await navigator.clipboard.writeText(content(selected, 'text'));
				notice = (m) => m.history.copied;
			} else {
				const path = await api.saveTranscript(
					content(selected, $format),
					transcriptFilename(new Date(selected.startedAt), $format)
				);
				if (path) notice = (m) => `${m.transcript.savedTo} ${path}`;
			}
		} catch (e) {
			actionError = caught(e);
		} finally {
			busy = false;
		}
	}
	async function remove(id: string) {
		busy = true;
		actionError = '';
		notice = '';
		try {
			await sessionHistory.delete(id);
			if (selectedId === id) selectedId = '';
			confirmDelete = '';
			await refresh();
		} catch (e) {
			actionError = caught(e);
		} finally {
			busy = false;
		}
	}

	/** The language a saved line was captioned in, for its `lang` and `dir`: recorded on the
	 *  line in a session with two languages, otherwise the session's own — the caption
	 *  language, or the spoken one for subtitles when it was known. */
	function lineLanguage(session: SavedSession, line: SavedSession['lines'][number]) {
		if (line.language) return line.language;
		const lane = line.lane ?? 0;
		const captioned = lane === 1 ? session.secondTargetLanguage : session.targetLanguage;
		if (captioned) return captioned;
		return lane === 0 && session.sourceLanguage !== 'auto' ? session.sourceLanguage : undefined;
	}
</script>

<section class="history" aria-label={$t.history.heading}>
	<Preference
		label={$t.history.enable}
		note={$t.history.privacy}
		disabled={!desktop}
		bind:checked={$historyEnabled}
	/>
	<!-- The other place a transcript is kept, offered here as well as under the transcript,
	     which only shows once a session has left one (review U14). -->
	<RecoveryPreference onError={(message) => statusMessage.set(message)} />
	{#if !desktop}<p class="hint">{$t.design.desktopOnly}</p>{/if}

	{#if $historyError}
		<p role="alert">{$t.history.failed} {describeStatus($historyError, $t, $locale)}</p>
		<ToolButton disabled={busy || !$historyEnabled} onclick={() => sessionHistory.retry()}
			>{$t.history.retry}</ToolButton
		>
	{/if}
	<!-- Until there is a session to find, the search, the dates, the language filter and the
	     reader would be controls over nothing: the opt-ins and one sentence say all there is.
	     Refresh stays while a listing has failed, as the way to try again. The dialog keeps its
	     size whichever way this goes (`stableHeight`). -->
	{#if sessions.length || error}
		<ToolButton disabled={!desktop || busy} onclick={refresh}>{$t.history.refresh}</ToolButton>
	{/if}
	{#if !sessions.length}<p class="hint">{$t.history.empty}</p>{:else}
		<div class="filters">
			<Field label={$t.usability.search}><input type="search" bind:value={query} /></Field>
			<DateField label={$t.usability.from} bind:value={from} />
			<DateField label={$t.usability.to} bind:value={to} />
			<Field label={$t.usability.language}
				><Select bind:value={language}
					><option value="">{$t.usability.allLanguages}</option
					>{#each TARGET_LANGUAGES as code (code)}<option value={code}
							>{languageName(code, $locale)}</option
						>{/each}<option value="auto">{$t.usability.unknownLanguage}</option></Select
				></Field
			>
		</div>
		<ToolButton
			disabled={!query && !from && !to && !language}
			onclick={() => {
				query = '';
				from = '';
				to = '';
				language = '';
			}}>{$t.design.clearFilters}</ToolButton
		>
		{#if sessions.length && !filtered.length}<p>{$t.usability.noMatches}</p>{/if}
		<!-- Several sessions at once: tick them in the list, or every one the filters show, then
		     save them all to one folder or delete them all after one confirmation that counts
		     them. The reader beside the list keeps acting on the one session it shows. -->
		<div class="batch">
			<label class="select-all"
				><input
					type="checkbox"
					checked={allShown}
					indeterminate={chosen.length > 0 && !allShown}
					disabled={busy || !filtered.length}
					onchange={toggleAll}
				/>{$t.history.selectAll}</label
			>
			<span class="count" aria-live="polite">{$t.history.selected(chosen.length)}</span>
			{#if chosen.length}
				<ToolButton disabled={busy} onclick={() => checked.clear()}
					>{$t.history.clearSelection}</ToolButton
				>
				<ExportFormatSelect disabled={busy} label={$t.history.selectionFormat} />
				<ToolButton disabled={busy} onclick={() => void exportChosen()}
					>{$t.history.saveSelected}</ToolButton
				>
				<ConfirmButton
					variant="danger"
					confirming={confirmBatch}
					label={$t.history.deleteSelected}
					confirmLabel={$t.history.confirmDeleteSelected(chosen.length)}
					cancelLabel={$t.history.cancel}
					disabled={busy}
					onask={() => (confirmBatch = true)}
					onconfirm={() => void removeChosen()}
					oncancel={() => (confirmBatch = false)}
				/>
			{/if}
		</div>
		<div class="history-browser">
			<ul class="sessions">
				{#each filtered as entry (entry.id)}
					<li>
						<input
							type="checkbox"
							aria-label={$t.history.select(entryName(entry))}
							checked={checked.has(entry.id)}
							disabled={busy}
							onchange={(e) => toggle(entry.id, e.currentTarget.checked)}
						/>
						<ToolButton
							class="session"
							disabled={busy}
							aria-pressed={selectedId === entry.id}
							onclick={() => {
								selectedId = entry.id;
								title = entry.session?.title ?? '';
								confirmDelete = '';
								notice = '';
								// Its headline says "this session", so it goes with the selection.
								actionError = '';
							}}
						>
							{#if entry.session}
								<strong
									>{entry.session.title ||
										formatDateTime(entry.session.startedAt, $localeTag)}</strong
								>
								{#if entry.session.title}<span
										>{formatDateTime(entry.session.startedAt, $localeTag)}</span
									>{/if}
								<span
									>{formatElapsed(entry.session.durationMs)} · {entry.session.sourceLanguage ===
									'auto'
										? $t.history.auto
										: entry.session.sourceLanguage.toUpperCase()} → {captionLanguages(
										entry.session
									)}</span
								>
								{#if !entry.session.endedAt}<span>{$t.history.unfinished}</span>{/if}
							{:else}{$t.history.unreadable} <code>{entry.id}</code>{/if}
						</ToolButton>
					</li>
				{/each}
			</ul>
			<div class="detail">
				{#if selected}
					<form
						onsubmit={(e) => {
							e.preventDefault();
							void rename();
						}}
					>
						<Field label={$t.usability.title}
							><input maxlength="120" bind:value={title} disabled={busy} /></Field
						>
						<p class="hint">{$t.usability.titleHint}</p>
						<ToolButton type="submit" disabled={busy || title.trim() === (selected.title ?? '')}
							>{$t.usability.saveTitle}</ToolButton
						>
					</form>
					<div class="actions">
						<ToolButton disabled={busy} onclick={() => action('copy')}>{$t.history.copy}</ToolButton
						>
						<ExportFormatSelect disabled={busy} />
						<ToolButton
							disabled={busy || (isTimedFormat($format) && !timed)}
							onclick={() => action('export')}>{$t.transcript.saveAs}</ToolButton
						>
						<ConfirmButton
							variant="danger"
							confirming={confirmDelete === selectedId}
							label={$t.history.delete}
							confirmLabel={$t.history.confirmDelete}
							cancelLabel={$t.history.cancel}
							disabled={busy}
							onask={() => (confirmDelete = selectedId)}
							onconfirm={() => void remove(selectedId)}
							oncancel={() => (confirmDelete = '')}
						/>
					</div>
					{#if hasOriginalSpeech(selected.lines)}<Preference
							label={$t.transcript.includeOriginal}
							checked={$exportOriginal}
							disabled={busy}
							onchange={(value) => exportOriginal.set(value)}
						/>{/if}
					{#if isTimedFormat($format) && !timed}<p class="hint">
							{$t.transcript.noTiming}
						</p>{/if}
					<!-- svelte-ignore a11y_no_noninteractive_tabindex (Scrollable saved transcript.) -->
					<div class="saved-text" role="region" aria-label={$t.history.fullTranscript} tabindex="0">
						<!-- Each line in the language it was captioned in, as the live log does (review U12),
					     and the original speech once: under the first language's line only, as the
					     saved documents write it — the second language heard the same speech. The
					     original's language is not recorded, so it is `lang=""`, its direction taken
					     from the text. -->
						{#each readerItems(selected) as item (item.kind === 'marker' ? `m${item.marker.id}` : item.line.id)}
							{#if item.kind === 'marker'}
								<h3 class="marker">
									<span class="marker-time">{markerTime(item.marker.atMs)}</span>
									{markerTitle(item.marker, item.number, $t.marker.section)}
								</h3>
							{:else}
								{@const line = item.line}
								{@const language = lineLanguage(selected, line)}
								<p>
									<strong>{$t.transcript.side[line.origin]}</strong>
									<span lang={language ?? ''} dir={captionDirection(language)}>{line.text}</span>
								</p>
								{#if line.sourceText && (line.lane ?? 0) === 0}<p class="source" lang="" dir="auto">
										{line.sourceText}
									</p>{/if}
							{/if}
						{/each}
					</div>
				{:else if selectedId}
					<p class="hint">{$t.history.unreadable}</p>
					<ConfirmButton
						variant="danger"
						confirming={confirmDelete === selectedId}
						label={$t.history.delete}
						confirmLabel={$t.history.confirmDelete}
						cancelLabel={$t.history.cancel}
						disabled={busy}
						cancelDisabled={false}
						onask={() => (confirmDelete = selectedId)}
						onconfirm={() => void remove(selectedId)}
						oncancel={() => (confirmDelete = '')}
					/>
				{:else}<p class="hint">{$t.design.selectSession}</p>
				{/if}
			</div>
		</div>
	{/if}
	{#if error}<p role="alert">{$t.history.failed} {describeStatus(error, $t, $locale)}</p>{/if}
	{#if actionError}<p role="alert">
			{$t.history.actionFailed}
			{describeStatus(actionError, $t, $locale)}
		</p>{/if}
	{#if batchError}<p role="alert">
			{$t.history.batchFailed(batchError.failed, batchError.total)}
			{describeStatus(batchError.error, $t, $locale)}
		</p>{/if}
	<p class="notice" role="status">{notice ? notice($t, $locale) : ''}</p>
</section>

<style>
	.history {
		display: grid;
		gap: var(--space-3);
		min-width: 0;
	}
	.filters {
		display: grid;
		grid-template-columns: repeat(4, minmax(0, 1fr));
		gap: var(--space-2);
	}
	/* A label that wraps (the search one does in French) would stretch its neighbours and
	   push their controls off the line: the label row takes the slack and its text sits at
	   the bottom, so every control starts at the same height. */
	.filters > :global(.ui-field) {
		grid-template-rows: 1fr auto;
	}
	.filters > :global(.ui-field > :first-child) {
		align-self: end;
	}
	.history-browser {
		display: grid;
		grid-template-columns: minmax(11rem, 0.8fr) minmax(0, 1.4fr);
		gap: var(--space-4);
		align-items: start;
	}
	.sessions {
		list-style: none;
		padding: 0;
		margin: 0;
		max-height: 28rem;
		overflow: auto;
	}
	li {
		display: flex;
		gap: var(--space-2);
		align-items: flex-start;
		margin-bottom: var(--space-2);
	}
	/* The row's checkbox, on the line of the session's name inside the button beside it. */
	li > input,
	.select-all input {
		accent-color: var(--accent);
		flex: none;
	}
	li > input {
		margin-top: var(--space-3);
	}
	/* Beside the buttons rather than on a row of its own. */
	.batch :global(.ui-select) {
		flex: 0 1 12rem;
		width: auto;
		min-width: 9rem;
	}
	.batch {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2) var(--space-3);
		align-items: center;
	}
	.select-all {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
		cursor: pointer;
		color: var(--text-secondary);
		font-size: var(--type-small);
	}
	.select-all:has(input:disabled) {
		cursor: default;
	}
	.count {
		color: var(--text-muted);
		font-size: var(--type-small);
		font-variant-numeric: tabular-nums;
	}
	.sessions :global(.session) {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		flex: 1;
		min-width: 0;
		text-align: left;
		overflow-wrap: anywhere;
		line-height: var(--leading-body);
	}
	.sessions :global(.session span) {
		color: var(--text-muted);
		font-size: var(--type-caption);
	}
	.detail {
		min-width: 0;
	}
	form {
		display: grid;
		gap: var(--space-2);
		margin-bottom: var(--space-3);
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
		align-items: center;
	}
	.actions :global(.ui-select) {
		flex: 1;
		min-width: 9rem;
	}
	.hint,
	.source,
	.notice {
		color: var(--text-muted);
		font-size: var(--type-small);
		line-height: var(--leading-body);
		margin: 0;
		overflow-wrap: anywhere;
	}
	.saved-text {
		max-height: 24rem;
		overflow: auto;
		overflow-wrap: anywhere;
		margin-top: var(--space-4);
		border-top: 1px solid var(--line);
		font-size: var(--type-body);
		line-height: var(--leading-body);
	}
	.saved-text p {
		margin: var(--space-3) 0;
	}
	/* An operator's marker: a talk's start, in the accent, under a rule. */
	.saved-text .marker {
		margin: var(--space-4) 0 var(--space-2);
		padding-top: var(--space-2);
		border-top: 1px solid var(--line-strong);
		font-size: var(--type-body);
		font-weight: 600;
		color: var(--accent-soft);
	}
	.marker-time {
		margin-inline-end: var(--space-2);
		font-family: var(--font-mono);
		font-weight: 500;
		color: var(--text-muted);
		font-variant-numeric: tabular-nums;
	}
	/* Asked of the settings dialog, in `em`, for the same reason as the appearance layout. */
	@container dialog (max-width: 40em) {
		.filters {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}
		.history-browser {
			grid-template-columns: minmax(0, 1fr);
		}
		.sessions {
			max-height: 12rem;
		}
	}
</style>
