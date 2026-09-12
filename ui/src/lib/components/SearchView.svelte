<!-- Search results, grouped into artists, albums and tracks (SPEC section 5.3). -->
<script lang="ts">
	import { coverUrl, search, type SearchResults } from '$lib/backend';
	import { navigate } from '$lib/app.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { library, setQuery } from '$lib/library.svelte';
	import TrackList from './TrackList.svelte';

	const LIMIT = 200;
	/** Typing pauses this long before the search runs. */
	const DEBOUNCE_MS = 120;

	let results = $state<SearchResults | null>(null);

	$effect(() => {
		const query = library.query;
		void library.version;
		const timer = setTimeout(() => {
			search(query, LIMIT)
				.then((found) => {
					if (query === library.query) results = found;
				})
				.catch(() => {});
		}, DEBOUNCE_MS);
		return () => clearTimeout(timer);
	});

	const empty = $derived(
		results !== null &&
			results.tracks.length === 0 &&
			results.albums.length === 0 &&
			results.artists.length === 0
	);
</script>

<div class="search">
	{#if empty}
		<p class="muted note">{t('search.none')}</p>
	{:else if results}
		{#if results.artists.length > 0 || results.albums.length > 0}
			<div class="groups">
				{#if results.artists.length > 0}
					<section>
						<h2 class="label">{t('search.artists')}</h2>
						<div class="chips">
							{#each results.artists as artist (artist.name)}
								<button type="button" class="chip" onclick={() => setQuery(artist.name)}>
									{artist.name} <span class="numeric muted">{artist.trackCount}</span>
								</button>
							{/each}
						</div>
					</section>
				{/if}
				{#if results.albums.length > 0}
					<section>
						<h2 class="label">{t('search.albums')}</h2>
						<div class="albums">
							{#each results.albums.slice(0, 24) as album (album.id)}
								<button
									type="button"
									class="album"
									onclick={() => {
										setQuery('');
										navigate({ kind: 'album', id: album.id });
									}}
								>
									<span class="art">
										{#if album.coverId !== null}<img src={coverUrl(album.coverId, 128)} alt="" />{/if}
									</span>
									<span class="ellipsis">{album.title}</span>
									<span class="ellipsis muted">{album.albumArtist}</span>
								</button>
							{/each}
						</div>
					</section>
				{/if}
			</div>
		{/if}
		{#if results.tracks.length > 0}
			<section class="tracks">
				<h2 class="label">{t('search.tracks')}</h2>
				<div class="list">
					<TrackList source={{ kind: 'tracks', tracks: results.tracks }} />
				</div>
			</section>
		{/if}
	{/if}
</div>

<style>
	.search {
		display: flex;
		flex-direction: column;
		gap: 12px;
		height: 100%;
		min-height: 0;
		padding: 12px 4px 0;
	}

	.note {
		padding: 12px;
	}

	.groups {
		display: grid;
		gap: 12px;
		padding: 0 10px;
	}

	h2 {
		margin: 0 0 6px;
		font-size: 11px;
		font-weight: 500;
	}

	.chips {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.albums {
		display: flex;
		gap: 10px;
		overflow-x: auto;
		padding-bottom: 4px;
	}

	.album {
		display: grid;
		gap: 2px;
		width: 104px;
		flex: none;
		padding: 0;
		border: 0;
		background: none;
		color: var(--onsa-text-primary);
		font: inherit;
		font-size: 12px;
		text-align: left;
		cursor: pointer;
	}

	.album .art {
		width: 104px;
		height: 104px;
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-well);
		overflow: hidden;
	}

	.album:hover .art {
		outline: var(--onsa-hairline) solid var(--onsa-role-adjustable);
	}

	.art img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}

	.tracks {
		display: grid;
		grid-template-rows: auto 1fr;
		flex: 1;
		min-height: 0;
	}

	.tracks h2 {
		padding: 0 10px;
	}

	.list {
		min-height: 0;
	}
</style>
