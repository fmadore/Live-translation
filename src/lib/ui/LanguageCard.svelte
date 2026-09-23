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
		gap: 0.5625rem;
		padding: 0.6875rem 0.75rem;
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		background: var(--panel-2);
		text-align: left;
		color: inherit;
	}
	.lang.selected {
		border-color: var(--accent-border);
		background: var(--accent-bg);
	}
	.lang:hover:not(:disabled) {
		border-color: var(--border-hover);
	}
	.lang.selected:hover:not(:disabled) {
		border-color: var(--accent);
	}
	.lang-code {
		font-family: var(--font-mono);
		font-size: var(--type-small);
		font-weight: 500;
		line-height: 1;
		color: var(--muted-2);
	}
	.lang.selected .lang-code {
		color: var(--accent-soft);
		font-weight: 600;
	}
	.lang-name {
		font-size: var(--type-body);
		font-weight: 500;
		line-height: 1;
		color: var(--text-soft);
	}
	.lang.selected .lang-name {
		color: var(--text);
		font-weight: 600;
	}
	@media (forced-colors: active) {
		.lang.selected {
			outline: 2px solid Highlight;
			outline-offset: -2px;
		}
	}
</style>
