<script lang="ts">
	import { formatDateTime, localeTag, t } from './i18n';
	import { isRunning } from './stores';
	import Preference from './ui/Preference.svelte';
	import UpdateNotice from './UpdateNotice.svelte';
	import type { UpdateCheck } from './updateCheck.svelte';

	/** Settings › App's Updates section (issue #217): the switch, the notice and when the last
	 *  check was made. Only for a copy installed from GitHub; a store build and a browser preview
	 *  show nothing. The notice waits while a session runs, as on the start screen. */
	let { updates }: { updates: UpdateCheck } = $props();
</script>

{#if updates.eligible}
	<div class="divider"></div>
	<div class="rail-section">
		<h2 class="kicker">{$t.updates.heading}</h2>
		<Preference
			label={$t.updates.toggle}
			note={$t.updates.toggleNote}
			checked={updates.checks === 'on'}
			onchange={(on) => void updates.setChecks(on)}
		/>
		{#if !$isRunning}<UpdateNotice {updates} place="settings" />{/if}
		<p class="hint">
			{updates.lastCheckedMs === null
				? $t.updates.notChecked
				: $t.updates.lastChecked(formatDateTime(updates.lastCheckedMs, $localeTag))}
		</p>
	</div>
{/if}
