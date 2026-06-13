<script lang="ts">
	import type { AudioLevel } from './types';

	let { level, label }: { level: AudioLevel; label: string } = $props();

	// Map RMS (0..1) to a percentage with a gentle curve so quiet speech is visible.
	const pct = $derived(Math.min(100, Math.round(Math.sqrt(level.rms) * 100)));
	const peakPct = $derived(Math.min(100, Math.round(Math.sqrt(level.peak) * 100)));
</script>

<div class="meter">
	<span class="label">{label}</span>
	<div class="track">
		<div class="fill" style="width: {pct}%"></div>
		<div class="peak" style="left: {peakPct}%"></div>
	</div>
</div>

<style>
	.meter {
		display: grid;
		grid-template-columns: 64px 1fr;
		align-items: center;
		gap: 8px;
	}
	.label {
		font-size: 12px;
		color: var(--muted);
	}
	.track {
		position: relative;
		height: 10px;
		border-radius: 5px;
		background: var(--panel-2);
		overflow: hidden;
		border: 1px solid var(--border);
	}
	.fill {
		position: absolute;
		left: 0;
		top: 0;
		bottom: 0;
		background: linear-gradient(90deg, var(--accent-2), var(--warn) 80%, var(--danger));
		transition: width 60ms linear;
	}
	.peak {
		position: absolute;
		top: 0;
		bottom: 0;
		width: 2px;
		background: var(--text);
		opacity: 0.7;
	}
</style>
