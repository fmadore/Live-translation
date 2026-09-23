<script lang="ts" generics="Id extends string">
	/**
	 * A tab list with a roving tab stop: Tab reaches only the selected tab, and Left/Right,
	 * Home and End move between tabs and select them. The panel is the caller's; each tab
	 * says it controls `panelId`, and the panel should be labelled by `tabId(selected)`.
	 */
	let {
		tabs,
		selected = $bindable(),
		label,
		idPrefix,
		panelId
	}: {
		tabs: readonly { id: Id; label: string }[];
		selected: Id;
		label: string;
		idPrefix: string;
		panelId: string;
	} = $props();

	const buttons: HTMLButtonElement[] = $state([]);

	function onkeydown(event: KeyboardEvent, index: number) {
		const last = tabs.length - 1;
		const next =
			event.key === 'ArrowRight'
				? index === last
					? 0
					: index + 1
				: event.key === 'ArrowLeft'
					? index === 0
						? last
						: index - 1
					: event.key === 'Home'
						? 0
						: event.key === 'End'
							? last
							: -1;
		if (next < 0) return;
		event.preventDefault();
		selected = tabs[next].id;
		buttons[next]?.focus();
	}
</script>

<div class="tabs" role="tablist" aria-label={label}>
	{#each tabs as tab, index (tab.id)}
		<button
			bind:this={buttons[index]}
			role="tab"
			id={`${idPrefix}-${tab.id}`}
			aria-selected={selected === tab.id}
			aria-controls={panelId}
			tabindex={selected === tab.id ? 0 : -1}
			onclick={() => (selected = tab.id)}
			onkeydown={(event) => onkeydown(event, index)}>{tab.label}</button
		>
	{/each}
</div>

<style>
	.tabs {
		display: flex;
		flex-wrap: wrap;
		gap: 0.375rem;
		border-bottom: 1px solid var(--border);
		padding-bottom: 0.75rem;
	}
	.tabs button {
		flex: 1;
		padding: 0.75rem;
		border: 1px solid transparent;
		border-radius: var(--radius-control);
		background: transparent;
		color: var(--muted);
		font-size: var(--type-small);
	}
	.tabs button[aria-selected='true'] {
		background: var(--accent-bg);
		color: var(--accent-soft);
		border-color: var(--accent-border);
	}
	.tabs button:hover {
		color: var(--text);
	}
	@media (forced-colors: active) {
		.tabs button[aria-selected='true'] {
			outline: 2px solid Highlight;
		}
	}
</style>
