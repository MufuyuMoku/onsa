<!-- Asks before something that cannot be taken back. -->
<script lang="ts">
	import { t } from '$lib/i18n/index.svelte';
	import Modal from './Modal.svelte';

	interface Props {
		title: string;
		/** What is about to happen, in a sentence. */
		message: string;
		/** The word on the button that goes ahead. */
		confirm: string;
		onconfirm: () => void;
		oncancel: () => void;
	}

	const { title, message, confirm, onconfirm, oncancel }: Props = $props();
</script>

<Modal {title} onclose={oncancel}>
	<p class="message">{message}</p>
	{#snippet footer()}
		<button type="button" class="btn" onclick={oncancel}>{t('playlist.cancel')}</button>
		<button type="button" class="btn caution" onclick={onconfirm}>{confirm}</button>
	{/snippet}
</Modal>

<style>
	.message {
		margin: 0;
		font-size: 13px;
		color: var(--onsa-text-primary);
	}

	.btn.caution {
		border-color: var(--onsa-role-caution);
		color: var(--onsa-role-caution);
	}
</style>
