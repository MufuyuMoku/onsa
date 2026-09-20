<!--
	Library folders: add one, rescan, see the last scan — and say so when a
	folder is not where it was put any more.
-->
<script lang="ts">
	import {
		addFolder,
		failureKey,
		libraryFolders,
		pickFolder,
		rescan,
		type LibraryFolder
	} from '$lib/backend';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';
	import { library, scanSummary } from '$lib/library.svelte';

	let folders = $state<LibraryFolder[]>([]);
	let failure = $state<MessageKey | null>(null);

	/** How often the disk is asked again while this page is open. */
	const AGAIN = 4000;

	function read(): void {
		libraryFolders()
			.then((list) => (folders = list))
			.catch((error) => (failure = failureKey(error)));
	}

	$effect(() => {
		void library.version;
		read();
	});

	// A drive can be pulled out while this page is on screen, and nothing
	// in the library changes when it happens — so the disk is asked again
	// every few seconds for as long as somebody is looking at this page.
	$effect(() => {
		const timer = setInterval(read, AGAIN);
		return () => clearInterval(timer);
	});

	const gone = $derived(folders.filter((one) => !one.present));

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
				<li class:gone={!folder.present}>
					<span class="ellipsis numeric">{folder.name}</span>
					{#if folder.present}
						<span class="muted numeric">{t('library.tracks', { n: folder.trackCount })}</span>
					{:else}
						<span class="caution-text numeric">{t('librarySettings.folderGone')}</span>
					{/if}
				</li>
			{/each}
		</ul>

		{#if gone.length > 0}
			<p class="note caution-text">{t('librarySettings.goneWhat', { n: gone.length })}</p>
			<p class="note muted">{t('librarySettings.goneNothingLost')}</p>
		{/if}
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
		<p class="note muted">{t('librarySettings.cannotRemove')}</p>
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

	li.gone .numeric:first-child {
		color: var(--onsa-role-caution);
	}

	.caution-text {
		color: var(--onsa-role-caution);
	}
</style>
