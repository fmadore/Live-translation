<script lang="ts">
	// Asked once for each cloud provider, before audio is first sent to it (App Review
	// Guideline 5.1.2(i); docs/decisions/2026-11-05-mac-app-store.md). It names the provider,
	// says what goes to it and under whose account, and gives the privacy policy's address.
	// Escape and Cancel start nothing; only Send records the answer.

	import ModalPrompt from './ModalPrompt.svelte';
	import ToolButton from './ui/ToolButton.svelte';
	import { t } from './i18n';
	import { PRIVACY_POLICY_URL } from './aiConsent';

	interface Props {
		/** The provider's name as the engine picker shows it. */
		vendor: string;
		onChoice: (send: boolean) => void;
	}

	let { vendor, onChoice }: Props = $props();

	let sendEl = $state<HTMLButtonElement | null>(null);

	$effect(() => {
		sendEl?.focus();
	});
</script>

<ModalPrompt title={$t.prompt.aiConsent.title(vendor)} onDismiss={() => onChoice(false)}>
	<p>{$t.prompt.aiConsent.body(vendor)}</p>
	<p class="note">{$t.prompt.aiConsent.note} <code>{PRIVACY_POLICY_URL}</code></p>

	<div class="actions">
		<ToolButton variant="primary" size="lg" bind:element={sendEl} onclick={() => onChoice(true)}>
			{$t.prompt.aiConsent.send(vendor)}
		</ToolButton>
		<ToolButton variant="ghost" size="lg" onclick={() => onChoice(false)}
			>{$t.prompt.aiConsent.cancel}</ToolButton
		>
	</div>
</ModalPrompt>
