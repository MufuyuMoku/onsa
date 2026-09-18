<!--
	One playlist (SPEC section 6).

	A manual playlist keeps the order the listener gave it, so its rows can
	be dragged and removed. A smart playlist is asked for again every time it
	is opened, so it follows the library and is read only here; its rules are
	what gets edited instead.
-->
<script lang="ts">
	import {
		failureKey,
		playlistExport,
		playlistTracks,
		type Rules,
		type Track
	} from '$lib/backend';
	import { navigate } from '$lib/app.svelte';
	import { clock, fileName } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';
	import { library } from '$lib/library.svelte';
	import { reorderable, reorderKey } from '$lib/reorder';
	import { addToQueue, playContext } from '$lib/player.svelte';
	import {
		deletePlaylist,
		duplicatePlaylist,
		moveInPlaylist,
		playlistById,
		playlists,
		removeFromPlaylist,
		renamePlaylist,
		setRules
	} from '$lib/playlists.svelte';
	import Confirm from './Confirm.svelte';
	import Icon from './Icon.svelte';
	import NamePrompt from './NamePrompt.svelte';
	import SmartRules from './SmartRules.svelte';
	import TrackList from './TrackList.svelte';

	interface Props {
		id: number;
	}

	const { id }: Props = $props();

	const playlist = $derived(playlistById(id));
	const smart = $derived(playlist?.kind === 'smart');

	let tracks = $state<Track[]>([]);
	let fault = $state<MessageKey | null>(null);
	let note = $state<string | null>(null);

	$effect(() => {
		// A smart playlist follows the library, so both versions matter.
		void library.version;
		void playlists.version;
		const wanted = id;
		playlistTracks(wanted)
			.then((rows) => {
				tracks = rows;
				fault = null;
			})
			.catch((error) => {
				fault = failureKey(error);
			});
	});

	/** Which panel is open, if any. */
	let asking = $state<'rename' | 'duplicate' | 'rules' | 'delete' | null>(null);
	let menuOpen = $state(false);

	function guard(work: Promise<unknown>): void {
		work.catch((error) => {
			fault = failureKey(error);
		});
	}

	function playFrom(index: number): void {
		guard(playContext({ kind: 'playlist', playlistId: id, index }));
	}

	async function doRename(name: string): Promise<void> {
		asking = null;
		guard(renamePlaylist(id, name));
	}

	async function doDuplicate(name: string): Promise<void> {
		asking = null;
		try {
			navigate({ kind: 'playlist', id: await duplicatePlaylist(id, name) });
		} catch (error) {
			fault = failureKey(error);
		}
	}

	async function doDelete(): Promise<void> {
		asking = null;
		try {
			await deletePlaylist(id);
			navigate({ kind: 'playlists' });
		} catch (error) {
			fault = failureKey(error);
		}
	}

	async function doRules(rules: Rules): Promise<void> {
		asking = null;
		guard(setRules(id, rules));
	}

	async function doExport(): Promise<void> {
		menuOpen = false;
		try {
			const path = await playlistExport(id, t('playlist.export'), t('playlist.fileFilter'));
			note = path === null ? null : t('playlist.exported', { path });
			fault = null;
		} catch (error) {
			fault = failureKey(error);
		}
	}

	// Rows are dragged with pointer events, the same way the queue's are.
	function reorder(from: number, to: number): void {
		guard(moveInPlaylist(id, from, to));
	}

	function shift(from: number, by: number): void {
		const to = from + by;
		if (to < 0 || to >= tracks.length) return;
		reorder(from, to);
	}
</script>

{#if !playlist}
	<p class="muted empty">{t('playlist.none')}</p>
{:else}
	<div class="playlist">
		<header class="head">
			<button
				type="button"
				class="icon-btn"
				aria-label={t('nav.playlists')}
				onclick={() => navigate({ kind: 'playlists' })}
			>
				<Icon name="back" />
			</button>
			<div class="who">
				<h1 class="ellipsis">{playlist.name}</h1>
				<p class="label">
					{smart ? t('playlist.smart') : t('playlist.manual')} ·
					<span class="numeric">{t('playlist.count', { n: tracks.length })}</span>
				</p>
			</div>
			<button
				type="button"
				class="icon-btn"
				aria-label={t('playlist.play')}
				title={t('playlist.play')}
				disabled={tracks.length === 0}
				onclick={() => playFrom(0)}
			>
				<Icon name="play" />
			</button>
			<button
				type="button"
				class="icon-btn"
				aria-label={t('queue.addToEnd')}
				title={t('queue.addToEnd')}
				disabled={tracks.length === 0}
				onclick={() => guard(addToQueue({ kind: 'playlist', playlistId: id, index: 0 }, 'end'))}
			>
				<Icon name="queue" />
			</button>
			<div class="menu-wrap">
				<button
					type="button"
					class="icon-btn"
					aria-label={t('playlist.more')}
					aria-expanded={menuOpen}
					onclick={() => (menuOpen = !menuOpen)}
				>
					<Icon name="more" />
				</button>
				{#if menuOpen}
					<!-- Clicking anywhere else puts the menu away. -->
					<div class="backdrop" role="presentation" onclick={() => (menuOpen = false)}></div>
					<menu class="menu">
						{#if smart}
							<li>
								<button
									type="button"
									onclick={() => {
										menuOpen = false;
										asking = 'rules';
									}}><Icon name="eq" />{t('playlist.editRules')}</button
								>
							</li>
						{/if}
						<li>
							<button
								type="button"
								onclick={() => {
									menuOpen = false;
									asking = 'rename';
								}}><Icon name="pencil" />{t('playlist.rename')}</button
							>
						</li>
						<li>
							<button
								type="button"
								onclick={() => {
									menuOpen = false;
									asking = 'duplicate';
								}}><Icon name="copy" />{t('playlist.duplicate')}</button
							>
						</li>
						<li>
							<button type="button" onclick={doExport}
								><Icon name="export" />{t('playlist.export')}</button
							>
						</li>
						<li>
							<button
								type="button"
								class="caution"
								onclick={() => {
									menuOpen = false;
									asking = 'delete';
								}}><Icon name="trash" />{t('playlist.delete')}</button
							>
						</li>
					</menu>
				{/if}
			</div>
		</header>

		{#if fault}
			<p class="fault-text notice">{t(fault)}</p>
		{:else if note}
			<p class="muted notice">{note}</p>
		{/if}

		<div class="body">
			{#if tracks.length === 0}
				<p class="muted empty">{smart ? t('playlist.emptySmart') : t('playlist.empty')}</p>
			{:else if smart}
				<TrackList source={{ kind: 'tracks', tracks }} view="playlist" />
			{:else}
				<ol
					class="rows"
					use:reorderable={{ count: () => tracks.length, move: reorder }}
				>
					{#each tracks as track, index (`${track.id}-${index}`)}
						<li>
							<div
								class="row listrow"
								class:alt={index % 2 === 1}
								class:dim={track.status !== 'ok'}
								data-index={index}
								role="button"
								tabindex="0"
								title={t('playlist.dragHint')}
								ondblclick={() => playFrom(index)}
								onkeydown={(event) => {
									if (reorderKey(event, index, tracks.length, reorder)) return;
									if (event.key === 'Enter') playFrom(index);
									if (event.key === 'Delete') guard(removeFromPlaylist(id, index));
								}}
							>
							<span class="numeric num">{index + 1}</span>
							<span class="text">
								<span class="ellipsis title">{track.title ?? fileName(track.path)}</span>
								<span class="ellipsis muted">{track.artist ?? t('track.unknownArtist')}</span>
							</span>
							<span class="numeric muted time"
								>{clock(track.durationMs === null ? null : track.durationMs / 1000)}</span
							>
							<span class="tools">
								<button
									type="button"
									aria-label={t('playlist.moveUp')}
									title={t('playlist.moveUp')}
									disabled={index === 0}
									onclick={() => shift(index, -1)}><Icon name="up" /></button
								>
								<button
									type="button"
									aria-label={t('playlist.moveDown')}
									title={t('playlist.moveDown')}
									disabled={index === tracks.length - 1}
									onclick={() => shift(index, 1)}><Icon name="down" /></button
								>
								<button
									type="button"
									aria-label={t('playlist.removeTrack')}
									title={t('playlist.removeTrack')}
									onclick={() => guard(removeFromPlaylist(id, index))}><Icon name="close" /></button
								>
								</span>
							</div>
						</li>
					{/each}
				</ol>
			{/if}
		</div>
	</div>

	{#if asking === 'rename'}
		<NamePrompt
			title={t('playlist.rename')}
			value={playlist.name}
			confirm={t('playlist.save')}
			onconfirm={doRename}
			oncancel={() => (asking = null)}
		/>
	{:else if asking === 'duplicate'}
		<NamePrompt
			title={t('playlist.duplicate')}
			value={t('playlist.copySuffix', { name: playlist.name })}
			confirm={t('playlist.create')}
			onconfirm={doDuplicate}
			oncancel={() => (asking = null)}
		/>
	{:else if asking === 'rules' && playlist.rules}
		<SmartRules
			title={t('rules.title')}
			value={playlist.rules}
			confirm={t('playlist.save')}
			onconfirm={doRules}
			oncancel={() => (asking = null)}
		/>
	{:else if asking === 'delete'}
		<Confirm
			title={t('playlist.delete')}
			message={t('playlist.deleteHint', { name: playlist.name })}
			confirm={t('playlist.delete')}
			onconfirm={doDelete}
			oncancel={() => (asking = null)}
		/>
	{/if}
{/if}

<style>
	.playlist {
		/* A column rather than fixed grid rows: the notice line is only
		   sometimes there, and the list has to take what is left either
		   way — as a row of its own it would collapse to its own height. */
		display: flex;
		flex-direction: column;
		min-height: 0;
		height: 100%;
	}

	.head {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 14px 16px 10px;
	}

	.who {
		flex: 1;
		min-width: 0;
	}

	h1 {
		margin: 0;
		font-size: 15px;
		font-weight: 500;
		color: var(--onsa-text-primary);
	}

	.who p {
		margin: 2px 0 0;
		font-size: 10.5px;
	}

	.menu-wrap {
		position: relative;
	}

	.backdrop {
		position: fixed;
		inset: 0;
		z-index: 10;
	}

	.menu {
		position: absolute;
		z-index: 11;
		top: calc(100% + 4px);
		right: 0;
		display: grid;
		margin: 0;
		padding: 4px;
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-raised);
		box-shadow: 0 10px 24px rgb(0 0 0 / 0.45);
		list-style: none;
	}

	.menu button {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 6px 14px 6px 10px;
		border: 0;
		border-radius: var(--onsa-radius-sm);
		background: none;
		color: var(--onsa-text-primary);
		font: inherit;
		font-size: 13px;
		white-space: nowrap;
		text-align: left;
		cursor: pointer;
	}

	.menu button:hover {
		background: var(--onsa-surface-body);
	}

	.menu button.caution {
		color: var(--onsa-role-caution);
	}

	.menu :global(svg) {
		width: 14px;
		height: 14px;
		flex: none;
	}

	.notice {
		margin: 0 16px 6px;
		font-size: 12px;
	}

	.empty {
		margin: 16px;
		font-size: 12px;
	}

	.body {
		display: grid;
		flex: 1;
		min-height: 0;
		overflow: hidden;
	}

	.rows {
		margin: 0;
		padding: 0 8px 12px;
		min-height: 0;
		overflow-y: auto;
		scrollbar-width: thin;
		list-style: none;
	}

	.row {
		display: grid;
		grid-template-columns: 34px minmax(0, 1fr) 52px auto;
		align-items: center;
		gap: 8px;
		padding: 5px 8px;
		border-radius: var(--onsa-radius-sm);
		border-top: var(--onsa-hairline) solid transparent;
		border-bottom: var(--onsa-hairline) solid transparent;
		cursor: default;
		user-select: none;
	}

	.row:hover {
		background: var(--onsa-surface-raised);
	}

	/* Where the row would land, drawn in the gap it is over. */
	.row:global([data-drop='before']) {
		border-top-color: var(--onsa-role-active);
	}

	.row:global([data-drop='after']) {
		border-bottom-color: var(--onsa-role-active);
	}

	.row:global([data-grabbed]) {
		opacity: 0.5;
	}

	/* While a row is being carried, nothing else takes the pointer. */
	.rows:global([data-reordering]) {
		user-select: none;
		cursor: grabbing;
	}

	.row.dim {
		color: var(--onsa-text-secondary);
	}

	.num {
		font-size: 11px;
		color: var(--onsa-text-secondary);
		text-align: right;
	}

	.text {
		display: grid;
		min-width: 0;
	}

	.title {
		font-size: 13px;
	}

	.text .muted {
		font-size: 11px;
	}

	.time {
		font-size: 11.5px;
		text-align: right;
	}

	.tools {
		display: flex;
		gap: 1px;
	}

	.tools button {
		display: grid;
		place-items: center;
		width: 24px;
		height: 24px;
		padding: 0;
		border: 0;
		border-radius: var(--onsa-radius-sm);
		background: none;
		color: var(--onsa-text-secondary);
		cursor: pointer;
	}

	.tools button:hover:not(:disabled) {
		color: var(--onsa-text-primary);
		background: var(--onsa-surface-body);
	}

	.tools button:disabled {
		opacity: 0.3;
		cursor: default;
	}

	.tools :global(svg) {
		width: 13px;
		height: 13px;
	}

	/* At the narrowest window the length gives way before the controls do. */
	@container content (max-width: 460px) {
		.row {
			grid-template-columns: 28px minmax(0, 1fr) auto;
		}

		.time {
			display: none;
		}
	}
</style>
