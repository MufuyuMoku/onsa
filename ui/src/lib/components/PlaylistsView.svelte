<!--
	The list of playlists (SPEC section 6): manual ones the listener filled
	themselves, smart ones that follow their rules, and the way in and out
	through M3U8 files.
-->
<script lang="ts">
	import { playlistImport, type Rules } from '$lib/backend';
	import { navigate } from '$lib/app.svelte';
	import { failureKey } from '$lib/backend';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';
	import { createPlaylist, emptyRules, playlists } from '$lib/playlists.svelte';
	import Icon from './Icon.svelte';
	import NamePrompt from './NamePrompt.svelte';
	import SmartRules from './SmartRules.svelte';

	/** Which panel is open, if any. */
	let asking = $state<'manual' | 'smart' | null>(null);
	let note = $state<string | null>(null);
	let fault = $state<MessageKey | null>(null);

	function report(error: unknown): void {
		fault = failureKey(error);
		note = null;
	}

	async function makeManual(name: string): Promise<void> {
		asking = null;
		try {
			navigate({ kind: 'playlist', id: await createPlaylist(name) });
		} catch (error) {
			report(error);
		}
	}

	async function makeSmart(rules: Rules): Promise<void> {
		asking = null;
		try {
			navigate({ kind: 'playlist', id: await createPlaylist(t('playlist.newSmart'), rules) });
		} catch (error) {
			report(error);
		}
	}

	async function importFile(): Promise<void> {
		fault = null;
		try {
			const report = await playlistImport(t('playlist.import'), t('playlist.fileFilter'));
			if (!report) return;
			note =
				report.playlistId === null
					? t('playlist.importedNone')
					: t('playlist.imported', { n: report.found, name: report.name });
			if (report.missing.length > 0) {
				note = `${note} ${t('playlist.importMissing', { n: report.missing.length })}`;
			}
			if (report.playlistId !== null) navigate({ kind: 'playlist', id: report.playlistId });
		} catch (error) {
			fault = failureKey(error);
		}
	}
</script>

<div class="playlists">
	<header class="head">
		<h1>{t('nav.playlists')}</h1>
		<button type="button" class="chip" onclick={() => (asking = 'manual')}>
			<Icon name="plus" />
			{t('playlist.new')}
		</button>
		<button type="button" class="chip" onclick={() => (asking = 'smart')}>
			<Icon name="plus" />
			{t('playlist.newSmart')}
		</button>
		<button type="button" class="chip" onclick={importFile}>
			<Icon name="import" />
			{t('playlist.import')}
		</button>
	</header>

	{#if fault}
		<p class="fault-text notice">{t(fault)}</p>
	{:else if note}
		<p class="muted notice">{note}</p>
	{/if}

	{#if playlists.all.length === 0}
		<p class="muted empty">{t('playlist.none')}</p>
	{:else}
		<ul class="list">
			{#each playlists.all as playlist (playlist.id)}
				<li>
					<button
						type="button"
						class="entry"
						onclick={() => navigate({ kind: 'playlist', id: playlist.id })}
					>
						<Icon name={playlist.kind === 'smart' ? 'eq' : 'playlist'} />
						<span class="name ellipsis">{playlist.name}</span>
						<span class="kind label">
							{playlist.kind === 'smart' ? t('playlist.smart') : t('playlist.manual')}
						</span>
						<span class="count numeric muted">{t('playlist.count', { n: playlist.trackCount })}</span
						>
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</div>

{#if asking === 'manual'}
	<NamePrompt
		title={t('playlist.new')}
		value={t('playlist.untitled')}
		confirm={t('playlist.create')}
		onconfirm={makeManual}
		oncancel={() => (asking = null)}
	/>
{:else if asking === 'smart'}
	<SmartRules
		title={t('playlist.newSmart')}
		value={emptyRules()}
		confirm={t('playlist.create')}
		onconfirm={makeSmart}
		oncancel={() => (asking = null)}
	/>
{/if}

<style>
	.playlists {
		display: flex;
		flex-direction: column;
		min-height: 0;
		height: 100%;
		padding: 14px 16px;
		overflow-y: auto;
		scrollbar-width: thin;
	}

	.head {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
	}

	h1 {
		flex: 1 1 auto;
		min-width: 0;
		margin: 0 8px 0 0;
		font-size: 15px;
		font-weight: 500;
		color: var(--onsa-text-primary);
	}

	.chip {
		display: inline-flex;
		align-items: center;
		gap: 5px;
	}

	.chip :global(svg) {
		width: 13px;
		height: 13px;
	}

	.notice,
	.empty {
		margin: 14px 0 0;
		font-size: 12px;
	}

	.list {
		display: grid;
		gap: 2px;
		margin: 14px 0 0;
		padding: 0;
		list-style: none;
	}

	.entry {
		display: grid;
		grid-template-columns: auto minmax(0, 1fr) auto auto;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 8px 10px;
		border: 0;
		border-radius: var(--onsa-radius-sm);
		background: none;
		color: var(--onsa-text-primary);
		font: inherit;
		font-size: 13px;
		text-align: left;
		cursor: pointer;
	}

	.entry:hover {
		background: var(--onsa-surface-raised);
	}

	.entry :global(svg) {
		width: 15px;
		height: 15px;
		fill: var(--onsa-text-secondary);
	}

	.kind {
		font-size: 10.5px;
		white-space: nowrap;
	}

	.count {
		font-size: 12px;
		white-space: nowrap;
	}

	/* The narrowest window has room for the name and the count only. */
	@container content (max-width: 460px) {
		.entry {
			grid-template-columns: auto minmax(0, 1fr) auto;
		}

		.kind {
			display: none;
		}
	}
</style>
