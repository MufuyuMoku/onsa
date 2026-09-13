<!--
	A track list: the whole library (sorted, paged from the backend) or a
	given list (an album, search results). Double-click or Enter plays from
	that row, with the rest of the list as the queue.
-->
<script lang="ts">
	import {
		trackCount,
		tracksPage,
		type PlayContext,
		type QueuePlace,
		type SortKey,
		type Track
	} from '$lib/backend';
	import { clock } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import { measure } from '$lib/layout.svelte';
	import { library, sortBy } from '$lib/library.svelte';
	import { addToQueue, player, playContext } from '$lib/player.svelte';
	import { addToPlaylist, playlists } from '$lib/playlists.svelte';
	import VirtualList from './VirtualList.svelte';

	type Source =
		| { kind: 'library' }
		| { kind: 'album'; albumId: number; tracks: Track[] }
		| { kind: 'tracks'; tracks: Track[] };

	interface Props {
		source: Source;
		showAlbum?: boolean;
		showNumber?: boolean;
	}

	let { source, showAlbum = true, showNumber = false }: Props = $props();

	const ROW = 32;
	const sortable = $derived(source.kind === 'library');

	let total = $state(0);
	$effect(() => {
		if (source.kind !== 'library') return;
		void library.version;
		trackCount()
			.then((count) => (total = count))
			.catch(() => {});
	});
	const count = $derived(source.kind === 'library' ? total : source.tracks.length);

	const load = $derived.by(() => {
		if (source.kind === 'library') {
			const sort = library.sort;
			const descending = library.descending;
			return (offset: number, limit: number) => tracksPage(sort, descending, offset, limit);
		}
		const tracks = source.tracks;
		return (offset: number, limit: number) => Promise.resolve(tracks.slice(offset, offset + limit));
	});

	function contextFor(index: number): PlayContext {
		if (source.kind === 'library') {
			return { kind: 'library', sort: library.sort, descending: library.descending, index };
		}
		if (source.kind === 'album') {
			return { kind: 'album', albumId: source.albumId, index };
		}
		return { kind: 'tracks', ids: source.tracks.map((track) => track.id), index };
	}

	function activate(index: number): void {
		playContext(contextFor(index)).catch(() => {});
	}

	/** The right-click menu: one track, added where the listener asks. */
	let menu = $state<{ x: number; y: number; id: number } | null>(null);

	function openMenu(event: MouseEvent, track: Track): void {
		event.preventDefault();
		if (track.id < 0) return;
		menu = { x: event.clientX, y: event.clientY, id: track.id };
	}

	function addOne(place: QueuePlace): void {
		const chosen = menu;
		menu = null;
		if (!chosen) return;
		addToQueue({ kind: 'tracks', ids: [chosen.id], index: 0 }, place).catch(() => {});
	}

	/** The same track, into a playlist the listener already has. */
	function addOneTo(playlistId: number): void {
		const chosen = menu;
		menu = null;
		if (!chosen) return;
		addToPlaylist(playlistId, { kind: 'tracks', ids: [chosen.id], index: 0 }).catch(() => {});
	}

	// Only manual playlists take tracks: a smart one is its rules.
	const manual = $derived(playlists.all.filter((playlist) => playlist.kind === 'manual'));

	function fileName(path: string): string {
		return path.split(/[\\/]/).pop() ?? path;
	}

	type Column = {
		key: SortKey;
		label: 'column.title' | 'column.artist' | 'column.album' | 'column.year' | 'column.duration';
		/** Width the list needs before this column is worth drawing. */
		needs: number;
	};

	// Title and length always stay; the rest go in this order as the list
	// narrows — the year first, then the album, then the artist. A column is
	// dropped rather than squeezed: squeezed columns end up on top of each
	// other (SPEC section 9.2).
	const columns: Column[] = [
		{ key: 'title', label: 'column.title', needs: 0 },
		{ key: 'artist', label: 'column.artist', needs: 560 },
		{ key: 'album', label: 'column.album', needs: 740 },
		{ key: 'year', label: 'column.year', needs: 880 },
		{ key: 'duration', label: 'column.duration', needs: 0 }
	];

	/** Width of the list itself, which is what the columns have to fit in. */
	let width = $state(1000);

	const shown = $derived(
		columns.filter(
			(column) =>
				(showAlbum || column.key !== 'album') &&
				(column.key !== 'album' || width >= column.needs) &&
				width >= column.needs
		)
	);
	const has = $derived({
		artist: shown.some((column) => column.key === 'artist'),
		album: shown.some((column) => column.key === 'album'),
		year: shown.some((column) => column.key === 'year')
	});
	/** The grid the head and every row share, built from what is shown. */
	const template = $derived(
		[
			'44px',
			'minmax(0, 2.2fr)',
			has.artist ? 'minmax(0, 1.4fr)' : '',
			has.album ? 'minmax(0, 1.6fr)' : '',
			has.year ? '48px' : '',
			'56px'
		]
			.filter(Boolean)
			.join(' ')
	);
</script>

<div class="tracklist" use:measure={(seen) => (width = seen)} style:--columns={template}>
	<div class="head label" role="row">
		<span class="num">{t('column.number')}</span>
		{#each shown as column (column.key)}
			{#if sortable}
				<button
					type="button"
					class="sort"
					class:right={column.key === 'duration'}
					aria-pressed={library.sort === column.key}
					title={t('column.sortHint')}
					onclick={() => sortBy(column.key)}
				>
					{t(column.label)}
					{#if library.sort === column.key}<span aria-hidden="true"
							>{library.descending ? '▼' : '▲'}</span
						>{/if}
				</button>
			{:else}
				<span class:right={column.key === 'duration'}>{t(column.label)}</span>
			{/if}
		{/each}
	</div>

	<div class="body">
		{#if count === 0}
			<p class="muted empty">{t('library.empty')}</p>
		{:else}
			<VirtualList {count} rowHeight={ROW} {load} version={library.version} label={t('nav.tracks')}>
				{#snippet row(track: Track | undefined, index: number)}
					{#if track}
						<div
							class="row"
							class:playing={player.snapshot?.track?.id === track.id}
							class:dim={track.status !== 'ok'}
							role="button"
							tabindex="0"
							title={t('track.playHint')}
							ondblclick={() => activate(index)}
							oncontextmenu={(event) => openMenu(event, track)}
							onkeydown={(event) => {
								if (event.key === 'Enter') activate(index);
							}}
						>
							<span class="num numeric">{showNumber ? (track.trackNumber ?? '') : index + 1}</span>
							<span class="ellipsis title">
								{track.title ?? fileName(track.path)}
								{#if track.status === 'missing'}<em>{t('track.missing')}</em>{/if}
								{#if track.status === 'failed'}<em>{t('track.failed')}</em>{/if}
							</span>
							{#if has.artist}
								<span class="ellipsis muted">{track.artist ?? t('track.unknownArtist')}</span>
							{/if}
							{#if has.album}<span class="ellipsis muted">{track.album ?? ''}</span>{/if}
							{#if has.year}<span class="numeric muted">{track.year ?? ''}</span>{/if}
							<span class="numeric muted right"
								>{clock(track.durationMs === null ? null : track.durationMs / 1000)}</span
							>
						</div>
					{:else}
						<div class="row placeholder"></div>
					{/if}
				{/snippet}
			</VirtualList>
		{/if}
	</div>
</div>

{#if menu}
	<!-- Closing the menu is what the backdrop is for; it is not a control. -->
	<div
		class="backdrop"
		role="presentation"
		onclick={() => (menu = null)}
		oncontextmenu={(event) => {
			event.preventDefault();
			menu = null;
		}}
	></div>
	<menu class="menu" style:left="{menu.x}px" style:top="{menu.y}px">
		<li>
			<button type="button" onclick={() => addOne('next')}>{t('queue.playNext')}</button>
		</li>
		<li>
			<button type="button" onclick={() => addOne('end')}>{t('queue.addToEnd')}</button>
		</li>
		{#if manual.length > 0}
			<li class="heading label" role="presentation">{t('playlist.addTo')}</li>
			{#each manual as playlist (playlist.id)}
				<li>
					<button type="button" class="ellipsis" onclick={() => addOneTo(playlist.id)}
						>{playlist.name}</button
					>
				</li>
			{/each}
		{/if}
	</menu>
{/if}

<svelte:window
	onkeydown={(event) => {
		if (event.key === 'Escape') menu = null;
	}}
/>

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		z-index: 10;
	}

	.menu {
		position: fixed;
		z-index: 11;
		display: grid;
		margin: 0;
		padding: 4px;
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-raised);
		box-shadow: 0 10px 24px rgb(0 0 0 / 0.45);
		list-style: none;
	}

	.menu {
		max-height: min(60vh, 420px);
		max-width: 260px;
		overflow-y: auto;
		scrollbar-width: thin;
	}

	.menu .heading {
		margin-top: 4px;
		padding: 6px 10px 2px;
		border-top: var(--onsa-hairline) solid var(--onsa-surface-line);
		font-size: 10px;
	}

	.menu button {
		width: 100%;
		padding: 6px 14px 6px 10px;
		border: 0;
		border-radius: var(--onsa-radius-sm);
		background: none;
		color: var(--onsa-text-primary);
		font: inherit;
		font-size: 12.5px;
		text-align: left;
		white-space: nowrap;
		cursor: pointer;
	}

	.menu button:hover {
		background: var(--onsa-surface-body);
		color: var(--onsa-role-active);
	}

	.tracklist {
		display: grid;
		grid-template-rows: auto 1fr;
		height: 100%;
		min-height: 0;
	}

	.head,
	.row {
		display: grid;
		grid-template-columns: var(--columns);
		align-items: center;
		gap: 10px;
		padding: 0 14px;
	}

	/* Nothing in a row may grow past its column, headings included. */
	.head > *,
	.row > * {
		min-width: 0;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}

	.head {
		height: 30px;
		font-size: 11px;
		border-bottom: var(--onsa-hairline) solid var(--onsa-surface-line);
	}

	.sort {
		padding: 0;
		border: 0;
		background: none;
		color: inherit;
		font: inherit;
		letter-spacing: inherit;
		text-transform: inherit;
		text-align: left;
		cursor: pointer;
	}

	.sort.right {
		text-align: right;
	}

	.sort[aria-pressed='true'] {
		color: var(--onsa-role-adjustable);
	}

	.right {
		text-align: right;
	}

	.body {
		min-height: 0;
	}

	.row {
		height: 32px;
		font-size: 13px;
		border-radius: var(--onsa-radius-sm);
		cursor: default;
		user-select: none;
	}

	.row:hover {
		background: var(--onsa-surface-raised);
	}

	.row.playing .title,
	.row.playing .num {
		color: var(--onsa-role-position);
	}

	.row.dim {
		opacity: 0.5;
	}

	.row em {
		margin-left: 6px;
		font-style: normal;
		font-size: 11px;
		color: var(--onsa-role-caution);
	}

	.num {
		text-align: right;
		color: var(--onsa-text-secondary);
	}

	.empty {
		padding: 24px;
	}
</style>
