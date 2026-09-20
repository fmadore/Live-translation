<script lang="ts">
	import type { HTMLButtonAttributes } from 'svelte/elements';
	let { children, class: className = '', ...props }: HTMLButtonAttributes = $props();
</script>

<button type="button" class="ui-tool {className}" {...props}>{@render children?.()}</button>
