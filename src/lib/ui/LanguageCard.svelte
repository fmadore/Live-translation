<script lang="ts">
	/** A language as a pressable card: its code and its name. Used for the interface language
	 *  and for the built-in demo's script language. */
	let {
		code,
		name,
		selected,
		disabled = false,
		onclick
	}: {
		/** Shown as written, so the caller decides the case. */
		code: string;
		name: string;
		selected: boolean;
		disabled?: boolean;
		onclick: () => void;
	} = $props();
</script>

<button class="lang" class:selected {disabled} aria-pressed={selected} {onclick}>
	<span class="lang-code">{code}</span>
	<span class="lang-name">{name}</span>
</button>

<style>
	.lang {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-3) var(--space-3);
		border: 1px solid var(--line-strong);
		border-radius: var(--radius-card);
		background: var(--surface-1);
		text-align: left;
		color: inherit;
	}
	.lang.selected {
		border-color: var(--accent-border);
		background: var(--accent-bg);
	}
	.lang:hover:not(:disabled) {
		border-color: var(--line-hover);
	}
	.lang.selected:hover:not(:disabled) {
		border-color: var(--accent);
	}
	.lang-code {
		font-family: var(--font-mono);
		font-size: var(--type-small);
		font-weight: 500;
		line-height: 1;
		color: var(--text-muted);
	}
	.lang.selected .lang-code {
		color: var(--accent-soft);
		font-weight: 600;
	}
	.lang-name {
		font-size: var(--type-body);
		font-weight: 500;
		line-height: 1;
		color: var(--text-secondary);
	}
	.lang.selected .lang-name {
		color: var(--text-body);
		font-weight: 600;
	}
	@media (forced-colors: active) {
		.lang.selected {
			outline: 2px solid Highlight;
			outline-offset: -2px;
		}
	}
</style>
