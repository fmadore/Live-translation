<script lang="ts">
	import { t } from './i18n';
	import SupportLink from './SupportLink.svelte';

	/** The idle stage's last line: who made the app, and the support link (issue #180).
	 *  Idle only, at the foot of the stage, so it never sits beside a running session. */
	const AUTHOR = 'Frédérick Madore';
</script>

<footer class="credit">
	<span>{$t.support.madeBy(AUTHOR)}</span>
	<span class="sep" aria-hidden="true">·</span>
	<SupportLink variant="inline" />
</footer>

<style>
	/* A direct child of the stage's column. Its automatic top margin takes whatever height is
	   left, so the line sits at the foot of a short stage and after the notes on a long one. */
	.credit {
		flex: 0 0 auto;
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
		margin-top: auto;
		padding-top: var(--space-6);
		font-size: var(--type-small);
		line-height: var(--leading-body);
		color: var(--text-muted);
	}
</style>
