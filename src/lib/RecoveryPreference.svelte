<script lang="ts">
	import Preference from './ui/Preference.svelte';
	import { isTauri } from './tauri';
	import { recovery } from './recovery';
	import { t } from './i18n';
	import { asStatus, type AppError } from './errors';
	import { recoveryEnabled } from './stores';

	/** The crash-recovery opt-in (issue #25): a local copy of the unsaved transcript, kept until
	 *  it is saved. Offered under the transcript and in Settings → History, so it can be chosen
	 *  before a first session has left anything to show (review U14). */
	let { onError }: { onError: (message: string | AppError) => void } = $props();

	// A browser preview has no filesystem to spool to, and inviting the operator to switch on a
	// feature whose only outcome would be an IPC error is the mistake issue #29 fixed elsewhere.
	const desktop = isTauri();

	function toggle(enabled: boolean) {
		if (!desktop) return;
		recoveryEnabled.set(enabled);
		// Switching it off has to remove what it already wrote, or "disabled" would still
		// leave captions sitting on disk.
		if (!enabled) void recovery.clear().catch((error) => onError(asStatus(error)));
	}
</script>

<Preference
	label={$t.transcript.recovery.title}
	note={desktop ? $t.transcript.recovery.note : $t.transcript.recovery.needsDesktop}
	checked={$recoveryEnabled}
	disabled={!desktop}
	onchange={toggle}
/>
