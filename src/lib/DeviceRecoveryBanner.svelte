<script lang="ts">
	import { t } from './i18n';
	import { options } from './stores';
	import type { Origin } from './types';

	let {
		failed,
		busy,
		onRetry,
		onFallback,
		onReselect
	}: {
		/** The source whose capture device failed. */
		failed: Origin;
		busy: boolean;
		/** Start again on the same device. */
		onRetry: () => void;
		/** Start again on the Windows default device. */
		onFallback: () => void;
		/** Pick the captured application again; its process has gone. */
		onReselect: () => void;
	} = $props();

	const application = $derived(
		failed === 'system' && $options.systemCapture?.kind === 'application'
	);
</script>

<section class="device-recovery" aria-label={$t.devices.retry}>
	<p class="hint">
		<strong>{failed === 'microphone' ? $t.source.microphone : $t.source.system}</strong>: {application
			? $t.applications.recovery
			: $t.devices.recovery}
	</p>
	<button class="tool" disabled={busy} onclick={onRetry}>{$t.devices.retry}</button>
	<button class="tool" disabled={busy} onclick={application ? onReselect : onFallback}
		>{application ? $t.applications.reselect : $t.devices.fallback}</button
	>
</section>

<style>
	.device-recovery {
		padding: 0.75rem 1.25rem;
		display: flex;
		flex-wrap: wrap;
		gap: 0.625rem;
		border-bottom: 1px solid var(--line-strong);
	}
	.device-recovery p {
		flex-basis: 100%;
	}
</style>
