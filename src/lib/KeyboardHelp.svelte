<script lang="ts">
	import { t } from './i18n';
	import Kbd from './ui/Kbd.svelte';
</script>

<!-- Every key cap here is printed from `shortcuts.ts`, which is also what the window listens
     for, so the list cannot promise a key the window does not answer to. -->
<section class="keys">
	<h3>{$t.usability.shortcuts}</h3>
	<p>{$t.usability.shortcutHint}</p>
	<dl>
		<dt><Kbd command="toggleSession" /></dt>
		<dd>{$t.usability.shortcutStart}</dd>
		<dt><Kbd command="toggleOverlay" /></dt>
		<dd>{$t.usability.shortcutOverlay}</dd>
		<dt><Kbd command="larger" /> / <Kbd command="smaller" /></dt>
		<dd>{$t.usability.shortcutSize}</dd>
		<dt><Kbd command="direction" /></dt>
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
	.keys {
		margin: 1rem 0;
		font-size: var(--type-small);
		color: var(--text-secondary);
	}
	h3 {
		margin: 0;
		font-size: var(--type-small);
		font-weight: 600;
	}
	p,
	dd {
		color: var(--text-muted);
		line-height: 1.5;
	}
	dd {
		margin: 0.2rem 0 0.75rem;
	}
	dt {
		font-weight: 600;
		line-height: 2;
	}
</style>
