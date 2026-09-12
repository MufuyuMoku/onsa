<!--
	A track list: the whole library (sorted, paged from the backend) or a
	given list (an album, search results). Double-click or Enter plays from
	that row, with the rest of the list as the queue.
-->
<script lang="ts">
	import { trackCount, tracksPage, type SortKey, type Track } from '$lib/backend';
	import { clock } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import { library, sortBy } from '$lib/library.svelte';
	import { player, playContext } from '$lib/player.svelte';
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

	function activate(index: number): void {
		const context =
			source.kind === 'library'
				? ({ kind: 'library', sort: library.sort, descending: library.descending, index } as const)
				: source.kind === 'album'
					? ({ kind: 'album', albumId: source.albumId, index } as const)
					: ({ kind: 'tracks', ids: source.tracks.map((track) => track.id), index } as const);
		playContext(context).catch(() => {});
	}

	function fileName(path: string): string {
		return path.split(/[\\/]/).pop() ?? path;
	}

	const columns: { key: SortKey; label: 'column.title' | 'column.artist' | 'column.album' | 'column.year' | 'column.duration' }[] =
		[
			{ key: 'title', label: 'column.title' },
			{ key: 'artist', label: 'column.artist' },
			{ key: 'album', label: 'column.album' },
			{ key: 'year', label: 'column.year' },
			{ key: 'duration', label: 'column.duration' }
		];
	const shown = $derived(columns.filter((column) => showAlbum || column.key !== 'album'));
</script>

<div class="tracklist" class:no-album={!showAlbum}>
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
							<span class="ellipsis muted">{track.artist ?? t('track.unknownArtist')}</span>
							{#if showAlbum}<span class="ellipsis muted">{track.album ?? ''}</span>{/if}
							<span class="numeric muted">{track.year ?? ''}</span>
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

<style>
	.tracklist {
		display: grid;
		grid-template-rows: auto 1fr;
		height: 100%;
		min-height: 0;
	}

	.head,
	.row {
		display: grid;
		grid-template-columns: 48px minmax(0, 2.2fr) minmax(0, 1.4fr) minmax(0, 1.6fr) 56px 64px;
		align-items: center;
		gap: 12px;
		padding: 0 14px;
	}

	.no-album .head,
	.no-album .row {
		grid-template-columns: 48px minmax(0, 2.4fr) minmax(0, 1.6fr) 56px 64px;
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
