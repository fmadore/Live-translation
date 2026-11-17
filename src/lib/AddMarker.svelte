<script lang="ts">
	import { get } from 'svelte/store';
	import { tick, untrack } from 'svelte';
	import { t } from './i18n';
	import { ariaKeyShortcut } from './shortcuts';
	import { addMarker, sessionStartedAt } from './stores';
	import { MARKER_LABEL_MAX } from './document';
	import { markerTime, markerTitle } from './transcript';
	import ToolButton from './ui/ToolButton.svelte';

	/** Add marker, in the transcript's toolbar while a session runs: a talk starting, a
	 *  speaker's name, or a blank, numbered section break, at the moment it is added. The
	 *  button opens a one-line field under the toolbar; Enter adds, Escape puts it away. */
	let {
		request = 0,
		count
	}: {
		/** Changes each time the window asks for the field (the shortcut). */
		request?: number;
		/** How many markers the document has, so a blank one is announced with its number. */
		count: number;
	} = $props();

	let open = $state(false);
	let label = $state('');
	let field = $state<HTMLInputElement | null>(null);
	let button = $state<HTMLButtonElement | null>(null);
	let announcement = $state('');
	// Where focus was when the field opened, to give it back.
	let returnTo: HTMLElement | null = null;
	const id = $props.id();

	async function show() {
		const active = document.activeElement;
		returnTo = active instanceof HTMLElement && active !== document.body ? active : null;
		open = true;
		await tick();
		field?.focus();
		field?.select();
	}

	let seen = untrack(() => request);
	$effect(() => {
		if (request === seen) return;
		seen = request;
		void show();
	});

	function close() {
		open = false;
		label = '';
		const target = returnTo?.isConnected && returnTo !== field ? returnTo : button;
		returnTo = null;
		void tick().then(() => target?.focus());
	}

	function add() {
		const started = get(sessionStartedAt);
		const marker = addMarker(label, started === null ? 0 : Date.now() - started);
		announcement = $t.marker.added(
			markerTitle(marker, count + 1, $t.marker.section),
			markerTime(marker.atMs)
		);
		close();
	}
</script>

<ToolButton
	variant="ghost"
	size="sm"
	bind:element={button}
	aria-expanded={open}
	aria-controls={open ? `${id}-form` : undefined}
	aria-keyshortcuts={ariaKeyShortcut('addMarker')}
	title={$t.marker.addLabel}
	onclick={() => (open ? close() : void show())}
>
	{$t.marker.add}
</ToolButton>
<!-- Said once a marker is in. In the DOM from the start, so the first one is heard. -->
<p class="sr-only" role="status">{announcement}</p>

{#if open}
	<form
		class="marker-form"
		id="{id}-form"
		onsubmit={(event) => {
			event.preventDefault();
			add();
		}}
	>
		<label class="marker-field">
			<span>{$t.marker.field}</span>
			<input
				bind:this={field}
				bind:value={label}
				type="text"
				maxlength={MARKER_LABEL_MAX}
				placeholder={$t.marker.placeholder}
				aria-describedby="{id}-hint"
				autocomplete="off"
				onkeydown={(event) => {
					if (event.key === 'Escape') {
						event.preventDefault();
						event.stopPropagation();
						close();
					}
				}}
			/>
		</label>
		<ToolButton type="submit" size="sm" variant="primary">{$t.marker.confirm}</ToolButton>
		<ToolButton size="sm" variant="ghost" onclick={close}>{$t.marker.cancel}</ToolButton>
		<p class="hint" id="{id}-hint">{$t.marker.hint}</p>
	</form>
{/if}

<style>
	/* A row of its own under the toolbar, the whole width of it. */
	.marker-form {
		flex-basis: 100%;
		display: flex;
		flex-wrap: wrap;
		align-items: flex-end;
		gap: var(--space-2);
		padding: var(--space-3);
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
		background: var(--surface-1);
	}
	.marker-field {
		flex: 1 1 14em;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		font-size: var(--type-small);
		color: var(--text-secondary);
	}
	.marker-field input {
		font: inherit;
		min-height: var(--control-sm);
		color: var(--text-body);
		background: var(--surface-0);
		border: 1px solid var(--line-strong);
		border-radius: var(--radius-control);
		padding: var(--space-1) var(--space-2);
	}
	.hint {
		flex-basis: 100%;
		margin: 0;
		font-size: var(--type-small);
		line-height: var(--leading-body);
		color: var(--text-muted);
	}
</style>
