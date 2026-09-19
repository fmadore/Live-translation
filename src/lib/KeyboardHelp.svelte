<script lang="ts">
	import { t } from './i18n';
</script>

<details class="keys">
	<summary>{$t.usability.shortcuts}</summary>
	<p>{$t.usability.shortcutHint}</p>
	<dl>
		<dt>Ctrl + Shift + Space</dt>
		<dd>{$t.usability.shortcutStart}</dd>
		<dt>Ctrl + Shift + O</dt>
		<dd>{$t.usability.shortcutOverlay}</dd>
		<dt>Ctrl + Shift + ↑ / ↓</dt>
		<dd>{$t.usability.shortcutSize}</dd>
		<dt>F2</dt>
		<dd>{$t.usability.shortcutDirection}</dd>
	</dl>
</details>

<style>
	.keys {
		margin: 1rem 0;
		font-size: var(--type-12);
		color: var(--text-soft);
	}
	summary {
		cursor: pointer;
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
