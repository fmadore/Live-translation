<script lang="ts">
	import { describeError } from './errors';
	import { t } from './i18n';
	import ToolButton from './ui/ToolButton.svelte';
	import type { UpdateCheck } from './updateCheck.svelte';

	/** The new-version notice (issue #217): "Version X.Y.Z is available", with Release notes and
	 *  Not now. On the start screen it is also where the question is asked, from the second
	 *  launch on, and its sentence is a live region; in Settings › App it is the notice alone.
	 *  Nothing at all for a copy that does not check (a store build, a browser preview). */
	let {
		updates,
		place = 'start'
	}: {
		updates: UpdateCheck;
		place?: 'start' | 'settings';
	} = $props();

	let error = $state<string | null>(null);

	const sentence = $derived(
		updates.available
			? $t.updates.available(updates.available)
			: place === 'start' && updates.ask
				? $t.updates.ask
				: ''
	);

	async function openNotes() {
		error = null;
		try {
			await updates.openReleaseNotes();
		} catch (failure) {
			error = describeError(failure, $t);
		}
	}
</script>

{#if updates.eligible && (place === 'start' || updates.available)}
	<div class={['update', place]}>
		<!-- Mounted with the window, before there is anything to say, so the sentence is heard
		     when it arrives. The buttons stay outside it: the region announces the news, not the
		     controls. -->
		<p class="sentence" role={place === 'start' ? 'status' : undefined}>{sentence}</p>
		{#if updates.available}
			<span class="actions">
				<ToolButton size="sm" variant="ghost" onclick={openNotes}
					>{$t.updates.releaseNotes}<span class="sr-only">{$t.support.opensInBrowser}</span
					></ToolButton
				>
				<ToolButton size="sm" variant="ghost" onclick={() => void updates.notNow()}
					>{$t.updates.notNow}</ToolButton
				>
			</span>
		{:else if place === 'start' && updates.ask}
			<span class="actions">
				<ToolButton size="sm" variant="ghost" onclick={() => void updates.setChecks(true)}
					>{$t.updates.turnOn}</ToolButton
				>
				<ToolButton size="sm" variant="ghost" onclick={() => void updates.setChecks(false)}
					>{$t.updates.noThanks}</ToolButton
				>
			</span>
		{/if}
		{#if error}<span class="failed" role="alert">{error}</span>{/if}
	</div>
{/if}

<style>
	/* One quiet line: the launch notes' size and grey, its buttons beside it. */
	.update {
		flex: 0 0 auto;
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		column-gap: var(--space-3);
		row-gap: var(--space-1);
		font-size: var(--type-small);
		line-height: var(--leading-body);
		color: var(--text-secondary);
	}
	.update.start {
		margin-top: var(--space-4);
	}
	/* Empty until there is news, and then it takes no room. Still displayed, so the live
	   region stays in the accessibility tree and is heard when its sentence arrives. */
	.update.start:has(.sentence:empty) {
		margin-top: 0;
	}
	.sentence {
		margin: 0;
		max-width: 72ch;
		text-wrap: pretty;
	}
	.actions {
		display: inline-flex;
		flex-wrap: wrap;
		gap: var(--space-2);
	}
	.failed {
		flex-basis: 100%;
		color: var(--warn);
	}
</style>
