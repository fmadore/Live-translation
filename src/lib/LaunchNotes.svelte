<script lang="ts">
	import { t } from './i18n';
	import { historyEnabled } from './history';
	import { isLocalWhisper } from './providers';
	import { options, recoveryEnabled, statusMessage } from './stores';

	/** The idle stage's foot: the status line, then what a rehearsal plays and where the
	 *  captions go — kept in memory, spooled for crash recovery, stored in history — and who
	 *  hears the audio. */
	let {
		statusText
	}: {
		/** The status line, as the page words it for its live region. */
		statusText: string;
	} = $props();
</script>

<!-- The visible copy of the page's status region, which is what a screen reader hears. -->
{#if $statusMessage}
	<p class="status-msg" aria-hidden="true">{statusText}</p>
{/if}

<div class="launch">
	<span class="rehearse-hint" id="rehearse-hint">
		{$options.provider === 'ondevice' ? $t.preflight.rehearse.demoHint : $t.preflight.rehearse.hint}
	</span>
	<span class="privacy">
		{$historyEnabled
			? $t.history.stored
			: $recoveryEnabled
				? $t.preflight.privacy.spooled
				: $t.preflight.privacy.memoryOnly}
		{$options.provider === 'ondevice'
			? $t.preflight.privacy.demo
			: isLocalWhisper($options.provider)
				? $t.whisper.privacy
				: $t.preflight.privacy.cloud($t.provider.vendor[$options.provider])}
	</span>
</div>

<style>
	/* Direct children of the stage's scrolling column: they keep their height rather than
	   compress, so a short window scrolls instead of clipping text. */
	.status-msg,
	.launch {
		flex: 0 0 auto;
	}
	.status-msg {
		margin: var(--space-4) 0 0;
		font-size: var(--type-body);
		line-height: var(--leading-body);
		color: var(--warn);
	}

	/* Keep actions aligned and let supporting text use the available reading width. */
	.launch {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: var(--space-3);
		margin-top: var(--space-6);
	}
	/* The two launch notes are one kind of text in the same grey, so they share a size: at two
	   sizes the second read as a separate, louder statement. */
	.rehearse-hint,
	.privacy {
		font-size: var(--type-small);
		line-height: var(--leading-body);
		color: var(--text-muted);
		max-width: 72ch;
		text-wrap: pretty;
	}

	/* Near the window's minimum height, tighten the vertical rhythm so the pre-flight checklist
	   and the Start button still land above the fold. */
	@media (max-height: 740px) {
		.launch {
			margin-top: var(--space-5);
		}
	}
</style>
