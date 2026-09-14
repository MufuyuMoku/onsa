<!-- One album: cover, details and its tracks in disc and track order. -->
<script lang="ts">
	import { albumTracks, coverUrl, type Track } from '$lib/backend';
	import { navigate } from '$lib/app.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { library } from '$lib/library.svelte';
	import { playContext } from '$lib/player.svelte';
	import Icon from './Icon.svelte';
	import TrackList from './TrackList.svelte';

	let { id }: { id: number } = $props();

	let tracks = $state<Track[]>([]);
	$effect(() => {
		void library.version;
		const album = id;
		albumTracks(album)
			.then((list) => {
				if (album === id) tracks = list;
			})
			.catch(() => {});
	});

	const first = $derived(tracks[0]);
</script>

<div class="album">
	<header class="head">
		<span class="art">
			{#if first?.coverId != null}<img src={coverUrl(first.coverId, 512)} alt="" />{/if}
		</span>
		<div class="text">
			<button type="button" class="back label" onclick={() => navigate({ kind: 'albums' })}>
				<Icon name="back" />{t('album.back')}
			</button>
			<h1 class="ellipsis">{first?.album ?? ''}</h1>
			<p class="ellipsis muted">
				{first?.albumArtist ?? first?.artist ?? ''}{first?.year ? ` · ${first.year}` : ''} ·
				{t('album.tracks', { n: tracks.length })}
			</p>
			<button
				type="button"
				class="btn primary"
				disabled={tracks.length === 0}
				onclick={() => playContext({ kind: 'album', albumId: id, index: 0 }).catch(() => {})}
			>
				{t('album.play')}
			</button>
		</div>
	</header>
	<div class="list">
		<TrackList source={{ kind: 'album', albumId: id, tracks }} view="album" showAlbum={false} showNumber />
	</div>
</div>

<style>
	.album {
		display: grid;
		grid-template-rows: auto 1fr;
		height: 100%;
		min-height: 0;
	}

	.head {
		display: grid;
		grid-template-columns: 150px minmax(0, 1fr);
		gap: 20px;
		padding: 18px;
	}

	.art {
		width: 150px;
		height: 150px;
		border-radius: var(--onsa-radius-md);
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
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		justify-content: flex-end;
		gap: 6px;
		min-width: 0;
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
		font-size: 24px;
		font-weight: 500;
	}

	p {
		margin: 0 0 6px;
		max-width: 100%;
	}

	.list {
		min-height: 0;
	}
</style>
