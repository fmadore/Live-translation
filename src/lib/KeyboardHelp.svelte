<script lang="ts">
	import { t } from './i18n';
</script>

<section class="keys">
	<h3>{$t.usability.shortcuts}</h3>
	<p>{$t.usability.shortcutHint}</p>
	<dl>
		<dt><kbd>Ctrl + Shift + Space</kbd></dt>
		<dd>{$t.usability.shortcutStart}</dd>
		<dt><kbd>Ctrl + Shift + O</kbd></dt>
		<dd>{$t.usability.shortcutOverlay}</dd>
		<dt><kbd>Ctrl + Shift + ↑ / ↓</kbd></dt>
		<dd>{$t.usability.shortcutSize}</dd>
		<dt><kbd>F2</kbd></dt>
		<dd>{$t.usability.shortcutDirection}</dd>
	</dl>
</section>

<style>
	dl {
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
		align-items: baseline;
		gap: 0.75rem;
	}
	kbd {
		font-family: var(--font-mono);
		font-size: var(--type-11);
		border: 1px solid var(--border);
		border-radius: var(--radius-control);
		padding: 0.25rem 0.375rem;
	}
	.keys {
		margin: 1rem 0;
		font-size: var(--type-12);
		color: var(--text-soft);
	}
	h3 {
		margin: 0;
		font-size: var(--type-12);
		font-weight: 600;
	}
	p,
	dd {
		color: var(--muted);
		line-height: 1.5;
	}
	dd {
		margin: 0.2rem 0 0.75rem;
	}
	dt {
		font-weight: 600;
	}
</style>
