<!-- Every album, paged from the backend; a click opens the album. -->
<script lang="ts">
	import { albumCount, albumsPage, coverUrl, type Album } from '$lib/backend';
	import { navigate } from '$lib/app.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { library } from '$lib/library.svelte';
	import VirtualList from './VirtualList.svelte';

	const ROW = 60;

	let total = $state(0);
	$effect(() => {
		void library.version;
		albumCount()
			.then((count) => (total = count))
			.catch(() => {});
	});
</script>

{#if total === 0}
	<p class="muted empty">{t('library.empty')}</p>
{:else}
	<VirtualList
		count={total}
		rowHeight={ROW}
		load={albumsPage}
		version={library.version}
		label={t('nav.albums')}
	>
		{#snippet row(album: Album | undefined)}
			{#if album}
				<button type="button" class="album" onclick={() => navigate({ kind: 'album', id: album.id })}>
					<span class="art">
						{#if album.coverId !== null}<img
								src={coverUrl(album.coverId, 128)}
								alt=""
								loading="lazy"
							/>{/if}
					</span>
					<span class="text">
						<span class="ellipsis title">{album.title}</span>
						<span class="ellipsis muted">
							{album.albumArtist || t('track.unknownArtist')}{album.year ? ` · ${album.year}` : ''}
						</span>
					</span>
					<span class="numeric muted count">{t('album.tracks', { n: album.trackCount })}</span>
				</button>
			{:else}
				<div class="album placeholder"></div>
			{/if}
		{/snippet}
	</VirtualList>
{/if}

<style>
	.album {
		display: grid;
		grid-template-columns: 48px minmax(0, 1fr) auto;
		align-items: center;
		gap: 14px;
		width: 100%;
		height: 60px;
		padding: 6px 14px;
		border: 0;
		border-radius: var(--onsa-radius-sm);
		background: none;
		color: var(--onsa-text-primary);
		font: inherit;
		text-align: left;
		cursor: pointer;
	}

	.album:hover {
		background: var(--onsa-surface-raised);
	}

	.art {
		width: 48px;
		height: 48px;
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-well);
		overflow: hidden;
	}

	.art img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}

	.text {
		display: grid;
		min-width: 0;
		font-size: 13px;
	}

	.title {
		font-size: 14px;
	}

	.count {
		font-size: 12px;
	}

	.empty {
		padding: 24px;
	}
</style>
