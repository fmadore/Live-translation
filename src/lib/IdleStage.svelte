<script lang="ts">
	import PreflightChecklist from './PreflightChecklist.svelte';
	import { t } from './i18n';

	/** The idle stage's head: what to check before the room arrives, and the checklist itself.
	 *  The saved transcript and the launch notes follow it on the page. */
	let {
		browserMode,
		locked,
		usesMic,
		usesSystem
	}: {
		browserMode: boolean;
		/** Setup is locked: a start or a profile load is under way. */
		locked: boolean;
		usesMic: boolean;
		usesSystem: boolean;
	} = $props();
</script>

<span class="kicker">{$t.preflight.kicker}</span>
<h2 class="ready">{$t.preflight.heading}</h2>
<p class="intro">{$t.preflight.intro}</p>

<PreflightChecklist {browserMode} {locked} {usesMic} {usesSystem} />

<style>
	/* Direct children of the stage's scrolling column: they keep their height rather than
	   compress, so a short window scrolls instead of clipping text. */
	.kicker,
	.ready,
	.intro {
		flex: 0 0 auto;
	}
	.ready {
		margin: var(--space-4) 0 0;
		font-size: var(--type-display);
		font-weight: 600;
		line-height: var(--leading-tight);
		letter-spacing: -0.02em;
	}
	.intro {
		margin: var(--space-2) 0 0;
		font-size: var(--type-body);
		line-height: var(--leading-body);
		color: var(--text-muted);
		max-width: 48ch;
		text-wrap: pretty;
	}

	/* Near the window's minimum height, tighten the vertical rhythm so the pre-flight checklist
	   and the Start button still land above the fold. */
	@media (max-height: 740px) {
		.ready {
			margin-top: var(--space-3);
			font-size: var(--type-heading);
		}
	}
</style>
