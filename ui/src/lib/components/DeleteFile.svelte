<!--
	Deleting a song's file (SPEC section 15).

	The panel asks the question only where the answer can be kept: Onsa
	checks the folder first, and a place with no Recycle Bin or Trash gets an
	explanation instead of a delete button. Nothing here can delete for good.
-->
<script lang="ts">
	import { deleteAllowed, deleteFiles, failureKey, type Kept } from '$lib/backend';
	import { fileName } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';
	import Modal from './Modal.svelte';

	interface Props {
		trackId: number;
		/** The file, as the library holds it. */
		path: string;
		onclose: () => void;
	}

	const { trackId, path, onclose }: Props = $props();

	/** Whether the folder can keep the promise; nothing until it is known. */
	let answer = $state<Kept | null>(null);
	let busy = $state(true);
	let done = $state<string | null>(null);
	let refused = $state<Kept | null>(null);
	let failure = $state<MessageKey | null>(null);

	const folder = $derived(path.slice(0, path.length - fileName(path).length).replace(/[\\/]+$/, ''));

	$effect(() => {
		let alive = true;
		deleteAllowed(folder)
			.then((said) => {
				if (alive) answer = said;
			})
			.catch((error) => {
				if (alive) failure = failureKey(error);
			})
			.finally(() => {
				if (alive) busy = false;
			});
		return () => {
			alive = false;
		};
	});

	/** The sentence that goes with a refusal, whichever kind it is. */
	function why(said: Kept): string {
		return said.reason === 'no_trash'
			? t('delete.noTrash')
			: said.reason === 'not_there'
				? t('delete.notThere')
				: t('delete.cannotTell');
	}

	async function go(): Promise<void> {
		busy = true;
		failure = null;
		try {
			const report = await deleteFiles([trackId]);
			if (report.deleted.length > 0) {
				// The lists reload on their own: deleting tells the interface
				// the library changed.
				done = report.deleted[0];
			} else {
				refused = report.kept[0] ?? null;
			}
		} catch (error) {
			failure = failureKey(error);
		} finally {
			busy = false;
		}
	}
</script>

<Modal title={t('delete.title')} {onclose}>
	{#if done}
		<p class="line">{t('delete.done', { name: fileName(done) })}</p>
		<p class="line muted">{t('delete.restoreHow')}</p>
	{:else if refused}
		<p class="line caution-text">{why(refused)}</p>
		{#if refused.detail}<p class="line muted numeric">{refused.detail}</p>{/if}
	{:else if answer && answer.reason !== 'ok'}
		<p class="line caution-text">{why(answer)}</p>
		{#if answer.detail}<p class="line muted numeric">{answer.detail}</p>{/if}
		<p class="line muted">{t('delete.neverForGood')}</p>
	{:else if answer}
		<p class="line">{t('delete.what', { name: fileName(path) })}</p>
		<p class="line muted">{t('delete.whereItGoes')}</p>
	{:else}
		<p class="line muted">{t('delete.checking')}</p>
	{/if}
	{#if failure}<p class="line fault-text">{t(failure)}</p>{/if}

	{#snippet footer()}
		{#if done || refused || (answer && answer.reason !== 'ok')}
			<button type="button" class="btn" onclick={onclose}>{t('editor.close')}</button>
		{:else}
			<button type="button" class="btn" onclick={onclose}>{t('playlist.cancel')}</button>
			<button
				type="button"
				class="btn caution"
				disabled={busy || answer === null}
				onclick={go}
			>
				{t('delete.confirm')}
			</button>
		{/if}
	{/snippet}
</Modal>

<style>
	.line {
		margin: 0 0 8px;
		font-size: 13px;
		color: var(--onsa-text-primary);
	}

	.line:last-child {
		margin-bottom: 0;
	}

	.caution-text {
		color: var(--onsa-role-caution);
	}

	.btn.caution {
		border-color: var(--onsa-role-caution);
		color: var(--onsa-role-caution);
	}
</style>
