<!--
	Browsing by artist, genre or folder (SPEC section 5.3): a paged list of
	names, then the tracks behind one of them.
-->
<script lang="ts">
	import {
		artistCount,
		artistsPage,
		artistTracks,
		directoriesPage,
		folderCount,
		folderTracks,
		genreCount,
		genresPage,
		genreTracks,
		type NameCount,
		type PlayContext,
		type Track
	} from '$lib/backend';
	import { navigate } from '$lib/app.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { library } from '$lib/library.svelte';
	import { playContext } from '$lib/player.svelte';
	import Icon from './Icon.svelte';
	import TrackList from './TrackList.svelte';
	import VirtualList from './VirtualList.svelte';

	type Kind = 'artists' | 'genres' | 'folders';

	interface Props {
		kind: Kind;
		/** Set on the detail view: the artist, genre or folder opened. */
		name?: string;
	}

	let { kind, name }: Props = $props();

	const ROW = 40;

	const counters = {
		artists: artistCount,
		genres: genreCount,
		folders: folderCount
	} as const;
	const pages = {
		artists: artistsPage,
		genres: genresPage,
		folders: directoriesPage
	} as const;
	const tracksOf = {
		artists: artistTracks,
		genres: genreTracks,
		folders: folderTracks
	} as const;
	const titles = { artists: 'nav.artists', genres: 'nav.genres', folders: 'nav.folders' } as const;

	let total = $state(0);
	let tracks = $state<Track[]>([]);

	$effect(() => {
		if (name !== undefined) return;
		void library.version;
		const of = kind;
		counters[of]()
			.then((count) => {
				if (of === kind) total = count;
			})
			.catch(() => {});
	});

	$effect(() => {
		const chosen = name;
		if (chosen === undefined) return;
		void library.version;
		tracksOf[kind](chosen)
			.then((found) => {
				if (chosen === name) tracks = found;
			})
			.catch(() => {});
	});

	function open(entry: NameCount): void {
		const to = kind === 'artists' ? 'artist' : kind === 'genres' ? 'genre' : 'folder';
		navigate(to === 'folder' ? { kind: 'folder', path: entry.name } : { kind: to, name: entry.name });
	}

	/** What playing from this artist, genre or folder means. */
	function context(index: number): PlayContext {
		const chosen = name ?? '';
		if (kind === 'artists') return { kind: 'artist', name: chosen, index };
		if (kind === 'genres') return { kind: 'genre', name: chosen, index };
		return { kind: 'folder', path: chosen, index };
	}

	/** Folders are shown by their own name, with the path beneath. */
	function leaf(path: string): string {
		return path.split(/[\\/]/).filter(Boolean).pop() ?? path;
	}
</script>

{#if name === undefined}
	{#if total === 0}
		<p class="muted empty">{t('library.empty')}</p>
	{:else}
		<VirtualList
			count={total}
			rowHeight={ROW}
			load={pages[kind]}
			version={library.version}
			label={t(titles[kind])}
		>
			{#snippet row(entry: NameCount | undefined)}
				{#if entry}
					<button type="button" class="entry" onclick={() => open(entry)}>
						<span class="text">
							<span class="ellipsis name" class:numeric={kind === 'folders'}>
								{kind === 'folders' ? leaf(entry.name) : entry.name}
							</span>
							{#if kind === 'folders'}
								<span class="ellipsis muted path numeric">{entry.name}</span>
							{/if}
						</span>
						<span class="numeric muted count">{t('library.tracks', { n: entry.trackCount })}</span>
					</button>
				{/if}
			{/snippet}
		</VirtualList>
	{/if}
{:else}
	<div class="detail">
		<header class="head">
			<button
				type="button"
				class="back label"
				onclick={() => navigate({ kind })}
			>
				<Icon name="back" />{t(titles[kind])}
			</button>
			<h1 class="ellipsis" class:numeric={kind === 'folders'}>
				{kind === 'folders' ? leaf(name) : name}
			</h1>
			<p class="muted ellipsis">
				{#if kind === 'folders'}<span class="numeric">{name}</span> · {/if}
				{t('library.tracks', { n: tracks.length })}
			</p>
			<button
				type="button"
				class="btn primary"
				disabled={tracks.length === 0}
				onclick={() => playContext(context(0)).catch(() => {})}
			>
				{t('transport.play')}
			</button>
		</header>
		<div class="list">
			<TrackList source={{ kind: 'tracks', tracks }} showAlbum={kind !== 'folders'} />
		</div>
	</div>
{/if}

<style>
	.entry {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		align-items: center;
		gap: 12px;
		width: 100%;
		height: 40px;
		padding: 0 14px;
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

	.text {
		display: grid;
		min-width: 0;
	}

	.path {
		font-size: 11px;
	}

	.count {
		font-size: 12px;
	}

	.empty {
		padding: 24px;
	}

	.detail {
		display: grid;
		grid-template-rows: auto 1fr;
		height: 100%;
		min-height: 0;
	}

	.head {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 6px;
		padding: 18px 18px 12px;
	}

	.back {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 0;
		border: 0;
		background: none;
		font-size: 11px;
		cursor: pointer;
	}

	.back:hover {
		color: var(--onsa-role-adjustable);
	}

	h1 {
		margin: 0;
		max-width: 100%;
		font-size: 22px;
		font-weight: 500;
	}

	p {
		margin: 0 0 6px;
		max-width: 100%;
		font-size: 12.5px;
	}

	.list {
		min-height: 0;
	}
</style>
