<script lang="ts">
	import { tick } from 'svelte';
	import { locale, t } from './i18n';
	import { languageName, languageRows, type TargetLanguage } from './languages';
	import type { Provider } from './types';
	let {
		value,
		provider,
		favourites,
		disabled = false,
		error = '',
		onchange,
		onpin
	}: {
		value: TargetLanguage;
		provider: Provider;
		favourites: TargetLanguage[];
		disabled?: boolean;
		error?: string;
		onchange: (code: TargetLanguage) => void;
		onpin: (code: TargetLanguage) => void;
	} = $props();
	const id = $props.id();
	let root: HTMLDivElement;
	let input: HTMLInputElement;
	let results = $state<HTMLDivElement>();
	let open = $state(false);
	let query = $state('');
	let edited = $state(false);
	let active = $state<TargetLanguage | undefined>();
	const rows = $derived(languageRows(provider, favourites, query, $locale));
	function gridRow(index: number) {
		return (
			index +
			1 +
			(query
				? 0
				: 1 + (rows[0]?.favourite && rows.slice(0, index + 1).some((r) => !r.favourite) ? 1 : 0))
		);
	}
	const display = $derived(`${value.toUpperCase()} · ${languageName(value, $locale)}`);
	const activeRow = $derived(rows.find((r) => r.code === active));
	$effect(() => {
		if (disabled) open = false;
		if (open && !rows.some((r) => r.code === active)) active = rows[0]?.code;
	});
	$effect(() => {
		if (open && active)
			document.getElementById(`${id}-${active}`)?.scrollIntoView?.({ block: 'nearest' });
	});
	async function show() {
		if (disabled || open) return;
		query = '';
		edited = false;
		active = value;
		open = true;
		await tick();
		input.select();
		results?.scrollIntoView?.({ block: 'nearest' });
	}
	function close() {
		open = false;
		query = '';
		edited = false;
	}
	function choose(code: TargetLanguage) {
		if (disabled || !rows.find((r) => r.code === code)?.supported) return;
		onchange(code);
		input.focus();
		close();
	}
	function keydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			event.preventDefault();
			close();
			return;
		}
		if (event.key === 'Enter') {
			event.preventDefault();
			if (!open) void show();
			else if (active) choose(active);
			return;
		}
		if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) return;
		if (!open) {
			if (event.key.startsWith('Arrow')) {
				event.preventDefault();
				void show();
			}
			return;
		}
		event.preventDefault();
		const index = rows.findIndex((r) => r.code === active);
		const next =
			event.key === 'Home'
				? 0
				: event.key === 'End'
					? rows.length - 1
					: Math.max(0, Math.min(rows.length - 1, index + (event.key === 'ArrowDown' ? 1 : -1)));
		active = rows[next]?.code;
	}
</script>

<svelte:window
	onpointerdown={(event) => {
		if (root && !root.contains(event.target as Node)) close();
	}}
/>
<div
	class="language-picker"
	bind:this={root}
	onfocusout={(event) => {
		if (!root.contains(event.relatedTarget as Node)) close();
	}}
>
	<input
		bind:this={input}
		role="combobox"
		aria-label={$t.usability.language}
		aria-expanded={open}
		aria-controls={`${id}-list`}
		aria-activedescendant={open && activeRow ? `${id}-${active}` : undefined}
		aria-autocomplete="list"
		aria-invalid={!!error}
		aria-describedby={`${id}-help${error ? ` ${id}-error` : ''}`}
		{disabled}
		value={open && edited ? query : display}
		autocomplete="off"
		spellcheck="false"
		onclick={show}
		onfocus={show}
		onkeydown={keydown}
		oninput={(event) => {
			open = true;
			edited = true;
			query = event.currentTarget.value;
			active = undefined;
		}}
	/>
	<p class="hint" id={`${id}-help`}>{$t.language.searchHint}</p>
	{#if error}<p class="error" id={`${id}-error`} role="status">{error}</p>{/if}
	{#if open}
		<div class="results" bind:this={results}>
			<!-- Pin buttons are siblings of the listbox, never interactive children of options. -->
			<div role="listbox" id={`${id}-list`} aria-label={$t.usability.language}>
				{#each rows as row, i (row.code)}
					{#if !query && (i === 0 || row.favourite !== rows[i - 1].favourite)}
						<div class="group" role="presentation" style:grid-row={gridRow(i) - 1}>
							{row.favourite ? `★ ${$t.language.favourites}` : $t.language.all}
						</div>
					{/if}
					<div
						class="option"
						class:active={active === row.code}
						class:unsupported={!row.supported}
						style:grid-row={gridRow(i)}
						id={`${id}-${row.code}`}
						role="option"
						aria-selected={row.code === value}
						aria-disabled={!row.supported}
						tabindex="-1"
						onpointerdown={(e) => e.preventDefault()}
						onclick={() => choose(row.code)}
						onkeydown={(e) => {
							if (e.key === 'Enter') choose(row.code);
						}}
					>
						<span><b>{row.code.toUpperCase()}</b> · {row.name}</span>
						{#if !row.supported}<small
								>{$t.language.unsupported($t.engine[provider], row.name)}</small
							>{/if}
					</div>
				{/each}
			</div>
			<div class="pins">
				{#each rows as row, i (row.code)}
					{#if !query && (i === 0 || row.favourite !== rows[i - 1].favourite)}<div
							class="group pin-group"
							style:grid-row={gridRow(i) - 1}
							aria-hidden="true"
						>
							&nbsp;
						</div>{/if}
					<div class="pin-row" style:grid-row={gridRow(i)}>
						<button
							type="button"
							tabindex={row.code === active ? 0 : -1}
							aria-pressed={row.favourite}
							aria-label={row.favourite ? $t.language.unpin(row.name) : $t.language.pin(row.name)}
							onclick={() => {
								onpin(row.code);
								input.focus();
							}}
							onkeydown={(e) => {
								if (e.key === 'Escape') {
									input.focus();
									close();
								}
							}}
						>
							{row.favourite ? '★' : '☆'}
						</button>
					</div>
				{/each}
			</div>
			{#if rows.length === 0}<p role="status">{$t.language.noMatches}</p>{/if}
		</div>
	{/if}
</div>

<style>
	.language-picker {
		min-width: 0;
	}
	input {
		width: 100%;
		min-width: 0;
		padding: 0.7em;
		color: var(--text);
		background: var(--panel);
		border: 1px solid var(--border);
		border-radius: 6px;
		font: inherit;
	}
	.hint,
	.error {
		margin: 0.5em 0;
		font-size: var(--type-12);
		overflow-wrap: anywhere;
	}
	.hint {
		color: var(--muted);
	}
	.error {
		color: var(--warn);
	}
	.results {
		display: grid;
		grid-template-columns: minmax(0, 1fr) 2.5em;
		max-height: min(20em, 45vh);
		overflow: auto;
		border: 1px solid var(--border);
		border-radius: 6px;
		background: var(--panel);
		scrollbar-color: var(--muted) var(--panel);
	}
	[role='listbox'],
	.pins {
		display: contents;
	}
	.option,
	.group {
		grid-column: 1;
	}
	.pin-row,
	.pin-group {
		grid-column: 2;
	}
	.group {
		padding: 0.5em;
		height: 2.5em;
		box-sizing: border-box;
		font-size: var(--type-12);
		color: var(--muted);
		border-top: 1px solid var(--border);
	}
	.option,
	.pin-row {
		min-height: 3em;
		box-sizing: border-box;
	}
	.option {
		display: flex;
		flex-direction: column;
		justify-content: center;
		padding: 0.6em;
		cursor: pointer;
		overflow-wrap: anywhere;
	}
	.option b {
		font-family: var(--font-mono);
		font-size: var(--type-12);
	}
	.option.active {
		background: var(--accent-bg);
		outline: 1px solid var(--accent);
		outline-offset: -1px;
	}
	.option:hover {
		background: var(--accent-bg);
	}
	.option.unsupported {
		cursor: default;
		color: var(--muted);
	}
	small {
		display: block;
		font-size: var(--type-12);
	}
	.pin-row {
		display: flex;
	}
	.pin-row button {
		display: grid;
		place-items: center;
		width: 100%;
		min-height: 3em;
		background: transparent;
		border: 0;
		color: var(--text);
		cursor: pointer;
	}
	.pin-row button:hover {
		color: var(--accent);
	}
	@media (forced-colors: active) {
		.option.active {
			outline-color: Highlight;
		}
	}
</style>
