<!-- Library folders: add one, rescan, see the last scan. -->
<script lang="ts">
	import { addFolder, failureKey, libraryFolders, pickFolder, rescan, type NameCount } from '$lib/backend';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';
	import { library, scanSummary } from '$lib/library.svelte';

	let folders = $state<NameCount[]>([]);
	let failure = $state<MessageKey | null>(null);

	$effect(() => {
		void library.version;
		libraryFolders()
			.then((list) => (folders = list))
			.catch((error) => (failure = failureKey(error)));
	});

	async function add(): Promise<void> {
		try {
			const picked = await pickFolder(t('firstRun.dialogTitle'));
			if (picked) await addFolder(picked);
			failure = null;
		} catch (error) {
			failure = failureKey(error);
		}
	}
</script>

<div class="page">
	<section>
		<h2 class="label">{t('librarySettings.folders')}</h2>
		<ul>
			{#each folders as folder (folder.name)}
				<li>
					<span class="ellipsis numeric">{folder.name}</span>
					<span class="muted numeric">{t('library.tracks', { n: folder.trackCount })}</span>
				</li>
			{/each}
		</ul>
		<div class="actions">
			<button type="button" class="btn" disabled={library.scan.running} onclick={add}>
				{t('librarySettings.add')}
			</button>
			<button
				type="button"
				class="btn"
				disabled={library.scan.running}
				onclick={() => rescan().catch((error) => (failure = failureKey(error)))}
			>
				{t('librarySettings.rescan')}
			</button>
		</div>
		<p class="note numeric">{scanSummary()}</p>
		{#if failure}<p class="note fault-text">{t(failure)}</p>{/if}
	</section>
</div>

<style>
	@import './settings.css';

	ul {
		display: grid;
		gap: 4px;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	li {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		gap: 12px;
		font-size: 13px;
	}
</style>
