<script lang="ts">
	import { localeTag, t } from './i18n';
	import { globalShortcuts } from './stores';
	import {
		ariaGlobalShortcut,
		globalKeyParts,
		globalShortcutStatus,
		refusedLabels,
		refusedPhrase
	} from './globalShortcuts';
	import { GLOBAL_SHORTCUTS } from './types';
	import Kbd from './ui/Kbd.svelte';
	import Preference from './ui/Preference.svelte';

	/** Settings › App, under the window's shortcuts: the opt-in system-wide Pause and Show /
	 *  hide overlay keys. The core holds them only while this is on and a session is running or
	 *  paused (`src-tauri/src/hotkeys.rs`); a combination another app holds is named below. */
	let { browserMode }: { browserMode: boolean } = $props();

	const refused = $derived($globalShortcuts ? refusedLabels($globalShortcutStatus, $t.keys) : []);
	const label = (shortcut: (typeof GLOBAL_SHORTCUTS)[number]) =>
		shortcut === 'toggle-pause' ? $t.usability.shortcutPause : $t.usability.shortcutOverlay;
</script>

<section class="global">
	<Preference
		label={$t.usability.globalShortcuts}
		note={$t.usability.globalShortcutsNote}
		checked={$globalShortcuts}
		disabled={browserMode}
		onchange={(value) => globalShortcuts.set(value)}
	/>
	<dl>
		{#each GLOBAL_SHORTCUTS as shortcut (shortcut)}
			<dt aria-keyshortcuts={ariaGlobalShortcut(shortcut)}>
				<Kbd parts={globalKeyParts(shortcut, $t.keys)} />
			</dt>
			<dd>{label(shortcut)}</dd>
		{/each}
	</dl>
	<!-- Always in the page, so a refusal is announced when it appears rather than missed
	     because the region arrived with it. -->
	<p class="refused" role="status">
		{#if refused.length}{$t.usability.globalRefused(
				refusedPhrase(refused, $localeTag),
				refused.length
			)}{/if}
	</p>
</section>

<style>
	.global {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		margin: 0 0 var(--space-4);
		font-size: var(--type-small);
		color: var(--text-secondary);
	}
	dl {
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
		align-items: baseline;
		gap: var(--space-3);
		margin: 0;
	}
	dt {
		font-weight: 600;
		line-height: var(--leading-body);
	}
	dd {
		margin: 0;
		color: var(--text-muted);
		line-height: var(--leading-body);
	}
	/* As in the shortcut list above: a cap may break after a `+` at large text sizes. */
	dt :global(.ui-kbd) {
		white-space: normal;
		box-decoration-break: clone;
	}
	/* Never `display: none` while empty: a status region that appears with its text is not
	   always announced. Empty, it takes no height. Two classes, to outrank the dialog's own
	   paragraph rule (`.prompt p`). */
	.global p.refused {
		margin: 0;
		font-size: var(--type-small);
		color: var(--warn-soft);
		line-height: var(--leading-body);
	}
</style>
