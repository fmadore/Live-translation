<script lang="ts">
	import { t } from './i18n';
	import { originStates, activityTimes, options, sessionStartedAt } from './stores';
	import { activity } from './liveActivity';
	import type { Origin } from './types';
	let { now, microphone, system }: { now: number; microphone: boolean; system: boolean } = $props();
	const sources = $derived(
		(['microphone', 'system'] as Origin[]).filter((o) => (o === 'microphone' ? microphone : system))
	);
</script>

<div class="activity" role="status" aria-label={$t.usability.activity}>
	{#each sources as origin}
		<p>
			<strong
				>{$options.provider === 'ondevice' ? $t.stage.origin.demo : $t.stage.origin[origin]}</strong
			>
			· {$t.usability[
				activity(
					$originStates[origin] ?? 'idle',
					now,
					$sessionStartedAt ?? now,
					$activityTimes[origin]?.audio ?? 0,
					$activityTimes[origin]?.caption ?? 0
				)
			]}
		</p>
	{/each}
</div>

<style>
	.activity {
		font-size: var(--type-12);
		line-height: 1.5;
		color: var(--text-soft);
	}
	p {
		margin: 0.5rem 0;
	}
</style>
