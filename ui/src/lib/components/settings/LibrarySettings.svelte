<!--
	Library folders: add one, let one go, follow one that moved, leave a
	subfolder out of scanning, rescan, see the last scan — and say so when a
	folder is not where it was put any more (SPEC section 15).
-->
<script lang="ts">
	import {
		addFolder,
		excludeFolder,
		exclusions,
		failureKey,
		includeFolder,
		libraryFolders,
		moveFolder,
		pickAnyFolder,
		pickFolder,
		removeFolder,
		rescan,
		type Exclusion,
		type LibraryFolder
	} from '$lib/backend';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';
	import { library, scanSummary } from '$lib/library.svelte';
	import Confirm from '../Confirm.svelte';

	let folders = $state<LibraryFolder[]>([]);
	let outOfScan = $state<Exclusion[]>([]);
	let failure = $state<MessageKey | null>(null);
	/** The folder somebody is being asked about letting go, if any. */
	let letting = $state<string | null>(null);

	/** How often the disk is asked again while this page is open. */
	const AGAIN = 4000;

	function read(): void {
		libraryFolders()
			.then((list) => (folders = list))
			.catch((error) => (failure = failureKey(error)));
		exclusions()
			.then((list) => (outOfScan = list))
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

	/** Runs something that changes the folders, and keeps the failure. */
	async function guard(work: () => Promise<unknown>): Promise<void> {
		try {
			await work();
			failure = null;
		} catch (error) {
			failure = failureKey(error);
		}
		read();
	}

	const add = () =>
		guard(async () => {
			const picked = await pickFolder(t('firstRun.dialogTitle'));
			if (picked) await addFolder(picked);
		});

	const followMoved = (from: string) =>
		guard(async () => {
			const picked = await pickAnyFolder(t('librarySettings.movedDialog'));
			if (picked) await moveFolder(from, picked);
		});

	const leaveOut = () =>
		guard(async () => {
			const picked = await pickAnyFolder(t('librarySettings.excludeDialog'));
			if (picked) await excludeFolder(picked);
		});

	const lookAgain = (path: string) => guard(() => includeFolder(path));

	function letGo(path: string): void {
		letting = null;
		void guard(() => removeFolder(path));
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
					<span class="row-actions">
						<button
							type="button"
							class="link"
							disabled={library.scan.running}
							onclick={() => followMoved(folder.name)}
						>
							{t('librarySettings.moved')}
						</button>
						<button
							type="button"
							class="link caution-text"
							disabled={library.scan.running}
							onclick={() => (letting = folder.name)}
						>
							{t('librarySettings.stopUsing')}
						</button>
					</span>
				</li>
			{/each}
		</ul>

		{#if gone.length > 0}
			<p class="note caution-text">{t('librarySettings.goneWhat', { n: gone.length })}</p>
			<p class="note muted">{t('librarySettings.goneNothingLost')}</p>
			<p class="note muted">{t('librarySettings.movedWhat')}</p>
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
		<p class="note numeric">{scanSummary()}</p>
		{#if failure}<p class="note fault-text">{t(failure)}</p>{/if}
	</section>

	<section>
		<h2 class="label">{t('librarySettings.excluded')}</h2>
		<p class="note muted">{t('librarySettings.excludedWhat')}</p>
		{#if outOfScan.length === 0}
			<p class="note muted">{t('librarySettings.excludedNone')}</p>
		{:else}
			<ul>
				{#each outOfScan as one (one.path)}
					<li class:gone={!one.present}>
						<span class="ellipsis numeric">{one.path}</span>
						{#if !one.present}
							<span class="caution-text numeric">{t('librarySettings.folderGone')}</span>
						{:else}
							<span></span>
						{/if}
						<span class="row-actions">
							<button
								type="button"
								class="link"
								disabled={library.scan.running}
								onclick={() => lookAgain(one.path)}
							>
								{t('librarySettings.lookAgain')}
							</button>
						</span>
					</li>
				{/each}
			</ul>
		{/if}
		<div class="actions">
			<button type="button" class="btn" disabled={library.scan.running} onclick={leaveOut}>
				{t('librarySettings.excludeAdd')}
			</button>
		</div>
	</section>
</div>

{#if letting}
	<Confirm
		title={t('librarySettings.stopTitle')}
		message={t('librarySettings.stopWhat', { folder: letting })}
		confirm={t('librarySettings.stopConfirm')}
		onconfirm={() => letGo(letting ?? '')}
		oncancel={() => (letting = null)}
	/>
{/if}

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
		grid-template-columns: minmax(0, 1fr) auto auto;
		align-items: baseline;
		gap: 12px;
		font-size: 13px;
	}

	li.gone .numeric:first-child {
		color: var(--onsa-role-caution);
	}

	.row-actions {
		display: flex;
		gap: 12px;
		white-space: nowrap;
	}

	.link {
		border: 0;
		background: none;
		padding: 0;
		font: inherit;
		color: var(--onsa-text-secondary);
		cursor: pointer;
		text-decoration: underline;
		text-underline-offset: 2px;
	}

	.link:hover:not(:disabled) {
		color: var(--onsa-text-primary);
	}

	.link:disabled {
		opacity: 0.5;
		cursor: default;
	}

	.caution-text {
		color: var(--onsa-role-caution);
	}
</style>
