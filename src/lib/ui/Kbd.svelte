<script lang="ts">
	import { t } from '../i18n';
	import { keyLabel, type Shortcut } from '../shortcuts';

	/** A shortcut as a key cap, in the interface language's key names. */
	let { command }: { command: Shortcut } = $props();
</script>

<kbd class="ui-kbd">{keyLabel(command, $t.keys)}</kbd>
