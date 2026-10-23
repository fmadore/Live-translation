<script lang="ts">
	import ChoiceButton from '../ui/ChoiceButton.svelte';
	import { t } from '../i18n';
	import { options } from '../stores';
	import type { SetupActions } from '../setupActions';

	/** Step 01: what the room reads — translated captions or subtitles in the spoken language. */
	let { actions, locked }: { actions: SetupActions; locked: boolean } = $props();
</script>

<section class="rail-section">
	<div class="step-head">
		<span class="step-no">01</span>
		<h2 class="kicker">{$t.rail.step.whatToShow}</h2>
	</div>
	<ChoiceButton
		selected={$options.mode === 'translate'}
		disabled={locked}
		title={$t.rail.translate.title}
		desc={$t.rail.translate.desc}
		onclick={() => actions.setMode('translate')}
	>
		{#snippet icon()}
			<svg
				width="18"
				height="18"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.75"
				stroke-linecap="round"><path d="M4 8.5h13l-3.5-3.5" /><path d="M20 15.5H7l3.5 3.5" /></svg
			>
		{/snippet}
	</ChoiceButton>
	<ChoiceButton
		selected={$options.mode === 'transcribe'}
		disabled={locked}
		title={$t.rail.transcribe.title}
		desc={$t.rail.transcribe.desc}
		onclick={() => actions.setMode('transcribe')}
	>
		{#snippet icon()}
			<svg
				width="18"
				height="18"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.75"
				stroke-linecap="round"><path d="M4 7h16M4 12h11M4 17h7" /></svg
			>
		{/snippet}
	</ChoiceButton>
</section>

<style>
	/* Direct children of the rail's scrolling column keep their height rather than
	   compress, so a short window scrolls instead of clipping text. */
	.kicker {
		flex: 0 0 auto;
	}
</style>
