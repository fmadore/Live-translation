<script lang="ts">
	import { t } from '../i18n';
	import { isMac } from '../platform';
	import { keyParts, type Shortcut } from '../shortcuts';

	/** A shortcut as a key cap, in the interface language's key names: one of the window's
	 *  (`command`), or keys already named (`parts`), as the system-wide ones are. */
	let { command, parts }: { command?: Shortcut; parts?: readonly string[] } = $props();
	// The label's keys, so a `<wbr>` can follow each `+`. A cap never wraps (`.ui-kbd`) unless
	// its container allows it, and then only between keys. A Mac writes its keys with nothing
	// between them (⇧⌘P).
	const keys = $derived(parts ?? (command ? keyParts(command, $t.keys) : []));
	const joiner = isMac ? '' : '+';
</script>

<kbd class="ui-kbd"
	>{#each keys as key, i (i)}{#if i}{joiner}<wbr />{/if}{key}{/each}</kbd
>
