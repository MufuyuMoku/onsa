<!--
	Every album as a grid of covers (Galeri, SPEC M16).

	The same paged reading as the album list: rows of covers are fetched as
	they come into view, a page at a time, so a library of ten thousand
	albums costs what is on screen and no more (SPEC section 13.1). How many
	covers fit in a row follows the width of the box the grid has.
-->
<script lang="ts">
	import { albumCount, albumsPage, coverUrl, type Album } from '$lib/backend';
	import { navigate } from '$lib/app.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { measure } from '$lib/layout.svelte';
	import { library } from '$lib/library.svelte';
	import VirtualList from './VirtualList.svelte';

	/** Smallest a tile may be, and the room its title takes under it. */
	const TILE = 170;
	const CAPTION = 52;
	const GAP = 16;

	let total = $state(0);
	let width = $state(800);
	$effect(() => {
		void library.version;
		albumCount()
			.then((count) => (total = count))
			.catch(() => {});
	});

	const columns = $derived(Math.max(1, Math.floor((width - GAP) / (TILE + GAP))));
	const tile = $derived(Math.floor((width - GAP * (columns + 1)) / columns));
	const rows = $derived(Math.ceil(total / columns));

	/** A page of rows is a page of albums cut into rows. */
	const load = $derived(async (offset: number, limit: number): Promise<Album[][]> => {
		const albums = await albumsPage(offset * columns, limit * columns);
		const cut: Album[][] = [];
		for (let at = 0; at < albums.length; at += columns) cut.push(albums.slice(at, at + columns));
		return cut;
	});
</script>

<div class="grid-box" use:measure={(seen) => (width = seen)}>
	{#if total === 0}
		<p class="muted empty">{t('library.empty')}</p>
	{:else}
		<VirtualList
			count={rows}
			rowHeight={tile + CAPTION + GAP}
			{load}
			version={`${library.version}:${columns}`}
			label={t('nav.albums')}
		>
			{#snippet row(albums: Album[] | undefined)}
				<div class="row" style:--tile="{tile}px" style:--columns={columns}>
					{#each albums ?? [] as album (album.id)}
						<button
							type="button"
							class="album"
							title={album.title}
							onclick={() => navigate({ kind: 'album', id: album.id })}
						>
							<span class="art">
								{#if album.coverId !== null}<img
										src={coverUrl(album.coverId, 512)}
										alt=""
										loading="lazy"
									/>{/if}
							</span>
							<span class="ellipsis title">{album.title}</span>
							<span class="ellipsis muted by">{album.albumArtist || t('track.unknownArtist')}</span>
						</button>
					{/each}
				</div>
			{/snippet}
		</VirtualList>
	{/if}
</div>

<style>
	.grid-box {
		height: 100%;
		min-width: 0;
	}

	.row {
		display: grid;
		grid-template-columns: repeat(var(--columns), var(--tile));
		gap: 16px;
		padding: 16px 16px 0;
	}

	.album {
		display: grid;
		grid-template-rows: var(--tile) auto auto;
		gap: 4px;
		min-width: 0;
		padding: 0;
		border: 0;
		background: none;
		color: var(--onsa-text-primary);
		font: inherit;
		text-align: left;
		cursor: pointer;
	}

	.art {
		width: var(--tile);
		height: var(--tile);
		border-radius: var(--onsa-radius-md);
		background: var(--onsa-surface-well);
		overflow: hidden;
		box-shadow: 0 6px 16px rgb(0 0 0 / 0.3);
	}

	.album:hover .art,
	.album:focus-visible .art {
		outline: 2px solid var(--onsa-role-adjustable);
		outline-offset: 2px;
	}

	.art img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}

	.title {
		font-size: 13px;
	}

	.by {
		font-size: 12px;
	}

	.empty {
		padding: 16px;
	}
</style>
