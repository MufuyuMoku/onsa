<!--
  The title bar of a window drawn inside the window (SPEC M16, Jendela
  Biru). It is a heading and nothing more: the operating system's own
  frame above it moves, shrinks, enlarges and closes the window, so this
  one has no buttons that would only pretend to.
-->
<script lang="ts">
	import { app } from '$lib/app.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';

	/** What each page is called, as the title says it. */
	const names: Partial<Record<string, MessageKey>> = {
		tracks: 'nav.tracks',
		albums: 'nav.albums',
		album: 'nav.albums',
		artists: 'nav.artists',
		artist: 'nav.artists',
		genres: 'nav.genres',
		genre: 'nav.genres',
		folders: 'nav.folders',
		folder: 'nav.folders',
		playlists: 'nav.playlists',
		playlist: 'nav.playlists',
		tidy: 'nav.tidy',
		downloads: 'nav.downloads',
		nowPlaying: 'nowPlaying.open',
		help: 'nav.help',
		settings: 'nav.settings'
	};

	const page = $derived(names[app.view.kind]);
</script>

<div class="titlebar">
	<span class="mark" aria-hidden="true"></span>
	<span class="title ellipsis">Onsa{page ? ` — ${t(page)}` : ''}</span>
</div>

<style>
	.titlebar {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
		height: 30px;
		padding: 0 8px;
		border-radius: 7px 7px 0 0;
		background: var(--onsa-chrome-title);
		/* The light caught along the top edge. */
		box-shadow: inset 0 2px 1px -1px rgb(255 255 255 / 0.45);
		color: var(--onsa-chrome-title-text);
	}

	.mark {
		flex: none;
		width: 16px;
		height: 16px;
		border-radius: 50%;
		background: radial-gradient(circle at 35% 30%, #fff, var(--onsa-lit-on) 60%, var(--onsa-chrome-frame));
	}

	.title {
		min-width: 0;
		font-size: 13px;
		font-weight: 700;
		text-shadow: 1px 1px 1px rgb(0 0 0 / 0.5);
	}
</style>
