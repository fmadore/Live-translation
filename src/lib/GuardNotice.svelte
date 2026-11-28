<script lang="ts">
	import { localeTag, t } from './i18n';
	import { formatUsd } from './providers';
	import type { GuardNotice, SpendingGuard } from './spendingGuard.svelte';
	import ToolButton from './ui/ToolButton.svelte';

	/** What the spending guard has to say, above the stage: why the session was paused, with
	 *  Resume, or that it has passed an amount or a length. Never a dialog: the session goes on
	 *  being what it was, and these wait to be read. Its status region says them. */
	let {
		guard,
		overlayNoticeShown,
		onResume,
		pausedNotice,
		onShowNotice
	}: {
		guard: SpendingGuard;
		/** The overlay already shows a notice, so the offer to put one up is not made. */
		overlayNoticeShown: boolean;
		onResume: () => void;
		/** The paused suggestion, in the language the room reads (`noticeLocale`). */
		pausedNotice: string;
		/** Put the paused notice on the overlay. */
		onShowNotice: () => void;
	} = $props();

	const notices = $derived(
		[
			guard.idlePaused !== null && {
				kind: 'idle' as GuardNotice,
				text: $t.guard.idlePaused(guard.idlePaused)
			},
			guard.costPassed !== null && {
				kind: 'cost' as GuardNotice,
				text: $t.guard.costPassed(formatUsd(guard.costPassed, $localeTag))
			},
			guard.durationPassed !== null && {
				kind: 'duration' as GuardNotice,
				text: $t.guard.durationPassed(guard.durationPassed)
			}
		].filter((notice) => notice !== false)
	);
</script>

<!-- What a screen reader hears: every notice up, in the words drawn below. Mounted with the
     session, minutes before the guard can have anything to say, so the first notice is heard;
     nothing is in it while the guard is quiet, which is always while it is switched off. -->
<p class="sr-only" role="status">{notices.map((notice) => notice.text).join(' ')}</p>

{#each notices as notice (notice.kind)}
	<!-- The status region above says these; this is their copy for the eye, with what can be
	     done about them. -->
	<section class="guard-notice" class:paused={notice.kind === 'idle'}>
		<p aria-hidden="true">{notice.text}</p>
		<div class="actions">
			{#if notice.kind === 'idle'}
				<ToolButton size="sm" variant="primary" onclick={onResume}>{$t.guard.resume}</ToolButton>
				{#if !overlayNoticeShown}
					<ToolButton size="sm" onclick={onShowNotice}>
						{$t.guard.showNotice(pausedNotice)}
					</ToolButton>
				{/if}
			{/if}
			<ToolButton size="sm" variant="ghost" onclick={() => guard.dismiss(notice.kind)}>
				{$t.guard.dismiss}
			</ToolButton>
		</div>
	</section>
{/each}

<style>
	.guard-notice {
		flex: 0 0 auto;
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2) var(--space-3);
		margin-bottom: var(--space-4);
		padding: var(--space-3);
		border: 1px solid var(--warn-border);
		border-radius: var(--radius-card);
		background: var(--warn-wash);
	}
	.guard-notice p {
		flex: 1 1 18em;
		margin: 0;
		font-size: var(--type-body);
		line-height: var(--leading-body);
		color: var(--warn-soft);
		text-wrap: pretty;
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
	}
</style>
