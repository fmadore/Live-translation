<script lang="ts">
	import { t } from './i18n';
	import { originState, originStates, activityTimes, options, runningSince } from './stores';
	import { activity } from './liveActivity';
	import { ORIGINS, type Origin } from './types';
	let { now, microphone, system }: { now: number; microphone: boolean; system: boolean } = $props();
	const sources = $derived(ORIGINS.filter((o) => (o === 'microphone' ? microphone : system)));
	const rows = $derived(
		sources.map((origin) => ({
			origin,
			activity: activity(
				originState($originStates, origin),
				now,
				$runningSince[origin] ?? now,
				$activityTimes[origin]?.audio ?? 0,
				$activityTimes[origin]?.caption ?? 0,
				$activityTimes[origin]?.silenced ?? 0
			)
		}))
	);
	function sourceName(origin: Origin): string {
		return $options.provider === 'ondevice' ? $t.stage.origin.demo : $t.stage.origin[origin];
	}
	// What a screen reader is told: only a source that has gone wrong — an error, audio with
	// no captions for 15 seconds, or sound playing that the system keeps from being recorded. The visible labels flip on every three-second gap in speech,
	// and a live region around them would read the panel out again each time. Empty otherwise,
	// so a source recovering is quiet, and going wrong again is a change that is announced.
	const alerts = $derived(
		rows
			.filter(
				(row) => row.activity === 'error' || row.activity === 'stale' || row.activity === 'silenced'
			)
			.map((row) => `${sourceName(row.origin)} · ${$t.usability[row.activity]}`)
			.join(' ')
	);
</script>

<div class="activity" role="group" aria-label={$t.usability.activity}>
	{#each rows as row (row.origin)}
		<p><strong>{sourceName(row.origin)}</strong> · {$t.usability[row.activity]}</p>
	{/each}
</div>
<!-- In the DOM from the first render, empty: a live region inserted with its text is often
     missed. -->
<p class="sr-only" role="status">{alerts}</p>

<style>
	.activity {
		font-size: var(--type-small);
		line-height: var(--leading-body);
		color: var(--text-secondary);
	}
	.activity p {
		margin: var(--space-2) 0;
	}
</style>
