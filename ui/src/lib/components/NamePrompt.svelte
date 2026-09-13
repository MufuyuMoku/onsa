<!-- Asks for one name: a new playlist, a rename, a copy. -->
<script lang="ts">
	import { t } from '$lib/i18n/index.svelte';
	import Modal from './Modal.svelte';

	interface Props {
		title: string;
		/** What the field starts with. */
		value?: string;
		/** The word on the button that goes ahead. */
		confirm: string;
		onconfirm: (name: string) => void;
		oncancel: () => void;
	}

	const { title, value = '', confirm, onconfirm, oncancel }: Props = $props();

	// The field starts from the name it was given and goes its own way from
	// there; later changes to the prop are not meant to overwrite typing.
	// svelte-ignore state_referenced_locally
	let name = $state(value);
	const ready = $derived(name.trim().length > 0);

	function go(): void {
		if (ready) onconfirm(name.trim());
	}
</script>

<Modal {title} onclose={oncancel}>
	<label class="row">
		<span class="label">{t('playlist.namePrompt')}</span>
		<input
			class="field"
			type="text"
			bind:value={name}
			onkeydown={(event) => {
				if (event.key === 'Enter') go();
			}}
		/>
	</label>
	{#snippet footer()}
		<button type="button" class="btn" onclick={oncancel}>{t('playlist.cancel')}</button>
		<button type="button" class="btn primary" disabled={!ready} onclick={go}>{confirm}</button>
	{/snippet}
</Modal>

<style>
	.row {
		display: grid;
		gap: 6px;
	}

	input {
		width: 100%;
	}
</style>
