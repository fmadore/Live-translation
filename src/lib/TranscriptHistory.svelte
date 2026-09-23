<script lang="ts">
	import { TARGET_LANGUAGES, languageName } from './languages';
	import Select from './ui/Select.svelte';
	import Field from './ui/Field.svelte';
	import DateField from './ui/DateField.svelte';
	import ToolButton from './ui/ToolButton.svelte';
	import Preference from './ui/Preference.svelte';
	import { matchesSession } from './historySearch';
	import { onMount } from 'svelte';
	import { api, isTauri } from './tauri';
	import { t, locale, localeTag, formatDateTime } from './i18n';
	import {
		decodeSession,
		historyEnabled,
		historyError,
		historyRevision,
		sessionHistory,
		type SavedSession
	} from './history';
	import {
		formatTranscript,
		transcriptFilename,
		hasTranscriptTiming,
		type TranscriptFormat
	} from './transcript';
	const desktop = isTauri();
	let sessions = $state<{ id: string; session: SavedSession | null }[]>([]);
	let selectedId = $state('');
	let busy = $state(false);
	let error = $state('');
	let notice = $state('');
	let confirmDelete = $state('');
	let format = $state<TranscriptFormat>('markdown');
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
	async function rename() {
		if (!selected || busy) return;
		busy = true;
		error = '';
		try {
			await sessionHistory.rename(selected, title);
			await refresh();
			notice = $t.usability.titleSaved;
		} catch (e) {
			error = String(e);
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
			const stored = await api.listHistory();
			if (current !== request) return;
			sessions = stored
				.map((s) => ({ id: s.path, session: decodeSession(s.contents, s.path) }))
				.sort((a, b) => (b.session?.startedAt ?? '').localeCompare(a.session?.startedAt ?? ''));
			error = '';
		} catch (e) {
			if (current === request) error = String(e);
		}
	}
	// A recording session writes history after every finalized line, and each write bumps the
	// revision. Re-listing every stored session for each one made an open History tab reread
	// the whole folder several times a minute, so the list catches up at most every few
	// seconds. Rename, delete and the Refresh button still refresh at once.
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

	function content(session: SavedSession, format: TranscriptFormat) {
		return formatTranscript(session.lines, format, new Date(session.startedAt), {
			title: $t.export.title,
			origin: $t.export.origin,
			tag: $localeTag
		});
	}
	async function action(kind: 'copy' | 'export') {
		if (!selected || busy) return;
		busy = true;
		error = '';
		notice = '';
		try {
			if (kind === 'copy') {
				await navigator.clipboard.writeText(content(selected, 'text'));
				notice = $t.history.copied;
			} else {
				const path = await api.saveTranscript(
					content(selected, format),
					transcriptFilename(new Date(selected.startedAt), format)
				);
				if (path) notice = `${$t.transcript.savedTo} ${path}`;
			}
		} catch (e) {
			error = String(e);
		} finally {
			busy = false;
		}
	}
	async function remove(id: string) {
		if (confirmDelete !== id) {
			confirmDelete = id;
			return;
		}
		busy = true;
		error = '';
		notice = '';
		try {
			await sessionHistory.delete(id);
			if (selectedId === id) selectedId = '';
			confirmDelete = '';
			await refresh();
		} catch (e) {
			error = String(e);
		} finally {
			busy = false;
		}
	}
</script>

<section class="history" aria-label={$t.history.heading}>
	<Preference
		label={$t.history.enable}
		note={$t.history.privacy}
		disabled={!desktop}
		bind:checked={$historyEnabled}
	/>
	{#if !desktop}<p class="hint">{$t.design.desktopOnly}</p>{/if}

	{#if $historyError}
		<p role="alert">{$t.history.failed} {$historyError}</p>
		<ToolButton disabled={busy || !$historyEnabled} onclick={() => sessionHistory.retry()}
			>{$t.history.retry}</ToolButton
		>
	{/if}
	<ToolButton disabled={!desktop || busy} onclick={refresh}>{$t.history.refresh}</ToolButton>
	{#if !sessions.length}<p class="hint">{$t.history.empty}</p>{/if}
	<div class="filters">
		<Field label={$t.usability.search}><input type="search" bind:value={query} /></Field>
		<DateField label={$t.usability.from} bind:value={from} />
		<DateField label={$t.usability.to} bind:value={to} />
		<Field label={$t.usability.language}
			><Select bind:value={language}
				><option value="">{$t.usability.allLanguages}</option
				>{#each TARGET_LANGUAGES as code}<option value={code}>{languageName(code, $locale)}</option
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
	<div class="history-browser">
		<ul class="sessions">
			{#each filtered as entry (entry.id)}
				<li>
					<ToolButton
						class="session"
						disabled={busy}
						aria-pressed={selectedId === entry.id}
						onclick={() => {
							selectedId = entry.id;
							title = entry.session?.title ?? '';
							confirmDelete = '';
							notice = '';
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
								>{Math.floor(entry.session.durationMs / 60000)}:{String(
									Math.floor(entry.session.durationMs / 1000) % 60
								).padStart(2, '0')} · {entry.session.sourceLanguage === 'auto'
									? $t.history.auto
									: entry.session.sourceLanguage.toUpperCase()} → {entry.session.targetLanguage?.toUpperCase() ??
									$t.history.sameLanguage}</span
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
					<ToolButton disabled={busy} onclick={() => action('copy')}>{$t.history.copy}</ToolButton>
					<Select aria-label={$t.transcript.format} bind:value={format} disabled={busy}>
						<option value="markdown">Markdown (.md)</option><option value="text"
							>{$t.transcript.plainText} (.txt)</option
						>
						<option value="vtt">WebVTT (.vtt)</option><option value="srt">SubRip (.srt)</option>
					</Select>
					<ToolButton
						disabled={busy || (['srt', 'vtt'].includes(format) && !timed)}
						onclick={() => action('export')}>{$t.transcript.saveAs}</ToolButton
					>
					<ToolButton disabled={busy} onclick={() => remove(selectedId)}
						>{confirmDelete === selectedId
							? $t.history.confirmDelete
							: $t.history.delete}</ToolButton
					>
					{#if confirmDelete === selectedId}<ToolButton
							disabled={busy}
							onclick={() => (confirmDelete = '')}>{$t.history.cancel}</ToolButton
						>{/if}
				</div>
				{#if ['srt', 'vtt'].includes(format) && !timed}<p class="hint">
						{$t.transcript.noTiming}
					</p>{/if}
				<!-- svelte-ignore a11y_no_noninteractive_tabindex (Scrollable saved transcript.) -->
				<div class="saved-text" role="region" aria-label={$t.history.fullTranscript} tabindex="0">
					{#each [...selected.lines].reverse() as line (line.id)}
						<p><strong>{$t.transcript.side[line.origin]}</strong> {line.text}</p>
						{#if line.sourceText}<p class="source">{line.sourceText}</p>{/if}
					{/each}
				</div>
			{:else if selectedId}
				<p class="hint">{$t.history.unreadable}</p>
				<ToolButton class="danger" disabled={busy} onclick={() => remove(selectedId)}
					>{confirmDelete === selectedId ? $t.history.confirmDelete : $t.history.delete}</ToolButton
				>
				{#if confirmDelete === selectedId}<ToolButton onclick={() => (confirmDelete = '')}
						>{$t.history.cancel}</ToolButton
					>{/if}
			{:else}<p class="hint">{$t.design.selectSession}</p>
			{/if}
		</div>
	</div>
	{#if error}<p role="alert">{$t.history.failed} {error}</p>{/if}
	<p class="notice" role="status">{notice}</p>
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
		margin-bottom: var(--space-2);
	}
	.sessions :global(.session) {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		width: 100%;
		text-align: left;
		overflow-wrap: anywhere;
		line-height: 1.5;
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
		line-height: 1.5;
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
		line-height: 1.6;
	}
	.saved-text p {
		margin: var(--space-3) 0;
	}
	@media (max-width: 720px) {
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
