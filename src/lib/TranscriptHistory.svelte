<script lang="ts">
	import { matchesSession } from './historySearch';
	import { onMount } from 'svelte';
	import { api, isTauri } from './tauri';
	import { t, localeTag, formatDateTime } from './i18n';
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
	let open = $state(false);
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
	onMount(() => () => {
		request++;
	});
	$effect(() => {
		if (open) {
			void $historyRevision;
			void refresh();
		}
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

<section class="history" class:open aria-label={$t.history.heading}>
	<div class="heading">
		<h2>{$t.history.heading}</h2>
		<button
			disabled={!desktop || busy}
			aria-expanded={open}
			onclick={() => {
				open = !open;
			}}>{open ? $t.history.close : $t.history.browse}</button
		>
	</div>
	<label
		><input type="checkbox" disabled={!desktop} bind:checked={$historyEnabled} />{$t.history
			.enable}</label
	>
	<p class="hint">{$t.history.privacy}</p>
	{#if $historyError}
		<p role="alert">{$t.history.failed} {$historyError}</p>
		<button disabled={busy || !$historyEnabled} onclick={() => sessionHistory.retry()}
			>{$t.history.retry}</button
		>
	{/if}
	{#if open}
		<button disabled={busy} onclick={refresh}>{$t.history.refresh}</button>
		{#if !sessions.length}<p class="hint">{$t.history.empty}</p>{/if}
		<div class="filters">
			<label>{$t.usability.search}<input type="search" bind:value={query} /></label>
			<label>{$t.usability.from}<input type="date" bind:value={from} /></label>
			<label>{$t.usability.to}<input type="date" bind:value={to} /></label>
			<label
				>{$t.usability.language}<select bind:value={language}
					><option value="">{$t.usability.allLanguages}</option><option value="en">English</option
					><option value="fr">Français</option><option value="auto"
						>{$t.usability.unknownLanguage}</option
					></select
				></label
			>
		</div>
		{#if sessions.length && !filtered.length}<p>{$t.usability.noMatches}</p>{/if}
		<ul class="sessions">
			{#each filtered as entry (entry.id)}
				<li>
					<button
						class="session"
						disabled={busy || !entry.session}
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
					</button>
					<button disabled={busy} onclick={() => remove(entry.id)}
						>{confirmDelete === entry.id ? $t.history.confirmDelete : $t.history.delete}</button
					>
					{#if confirmDelete === entry.id}<button
							disabled={busy}
							onclick={() => (confirmDelete = '')}>{$t.history.cancel}</button
						>{/if}
				</li>
			{/each}
		</ul>
		{#if selected}
			<form
				onsubmit={(e) => {
					e.preventDefault();
					void rename();
				}}
			>
				<label
					>{$t.usability.title}<input maxlength="120" bind:value={title} disabled={busy} /></label
				>
				<p class="hint">{$t.usability.titleHint}</p>
				<button disabled={busy || title.trim() === (selected.title ?? '')}
					>{$t.usability.saveTitle}</button
				>
			</form>
			<div class="actions">
				<button disabled={busy} onclick={() => action('copy')}>{$t.history.copy}</button>
				<select aria-label={$t.transcript.format} bind:value={format} disabled={busy}>
					<option value="markdown">Markdown (.md)</option><option value="text"
						>{$t.transcript.plainText} (.txt)</option
					>
					<option value="vtt">WebVTT (.vtt)</option><option value="srt">SubRip (.srt)</option>
				</select>
				<button
					disabled={busy || (['srt', 'vtt'].includes(format) && !timed)}
					onclick={() => action('export')}>{$t.transcript.saveAs}</button
				>
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
		{/if}
	{/if}
	{#if error}<p role="alert">{$t.history.failed} {error}</p>{/if}
	<p class="notice" role="status">{notice}</p>
</section>

<style>
	.filters {
		display: flex;
		flex-wrap: wrap;
		gap: 0.75rem;
	}
	.filters label,
	form label {
		display: grid;
		min-width: 0;
	}
	input {
		min-width: 0;
		max-width: 100%;
	}
	.history.open {
		max-height: 70vh;
		overflow: auto;
		flex-shrink: 0;
	}
	.history {
		color-scheme: dark;
		padding: 1rem;
		border-top: 1px solid var(--border);
		color: var(--text-soft);
	}
	.heading,
	.actions {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		flex-wrap: wrap;
	}
	h2 {
		font-size: var(--type-14);
		margin: 0;
		flex: 1;
	}
	label {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin-top: 0.75rem;
		font-size: var(--type-12);
	}
	.hint,
	.source {
		color: var(--muted);
		font-size: var(--type-12);
		line-height: 1.5;
	}
	button,
	input,
	select {
		background: var(--panel-2);
		border: 1px solid var(--border);
		color: var(--text-soft);
		border-radius: 8px;
		padding: 0.5rem 0.75rem;
		font-size: var(--type-12);
	}
	button:hover:not(:disabled) {
		border-color: var(--border-hover);
	}
	button[aria-pressed='true'] {
		border-color: var(--accent-border);
	}
	button:disabled {
		opacity: 0.5;
	}
	.sessions {
		list-style: none;
		padding: 0;
		max-height: 16rem;
		overflow: auto;
	}
	li {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 0;
		border-bottom: 1px solid var(--border);
	}
	.session {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 0.3rem;
		text-align: left;
		min-width: 12rem;
		max-width: 100%;
		overflow-wrap: anywhere;
	}
	.saved-text {
		max-height: 24rem;
		overflow: auto;
		overflow-wrap: anywhere;
		margin-top: 1rem;
		padding: 0.75rem;
		background: var(--panel-2);
		border-radius: 8px;
		font-size: var(--type-13);
		line-height: 1.6;
	}
	.saved-text p {
		margin: 0.25rem 0 0.75rem;
	}
	.notice {
		overflow-wrap: anywhere;
		font-size: var(--type-12);
	}
</style>
