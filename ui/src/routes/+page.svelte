<!--
	The window (SPEC section 9.2): sidebar, main area, queue, transport and
	the signal path strip. The first screen shows until the library has a
	folder.
-->
<script lang="ts">
	import { nextTrack, previousTrack, seek, togglePlay } from '$lib/backend';
	import { followTone } from '$lib/analysis.svelte';
	import { app, navigate, setMiniPlayer } from '$lib/app.svelte';
	import { carry } from '$lib/carry.svelte';
	import { closeOverlays, followWidth, layout, toggleMenu, toggleQueue } from '$lib/layout.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { library, setQuery } from '$lib/library.svelte';
	import { player } from '$lib/player.svelte';
	import { settings, updateDsp } from '$lib/settings.svelte';
	import { themeFailure } from '$lib/theme/index.svelte';
	import AlbumList from '$lib/components/AlbumList.svelte';
	import AlbumView from '$lib/components/AlbumView.svelte';
	import BrowseView from '$lib/components/BrowseView.svelte';
	import FirstRun from '$lib/components/FirstRun.svelte';
	import MiniPlayer from '$lib/components/MiniPlayer.svelte';
	import NowPlaying from '$lib/components/NowPlaying.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import PlaylistsView from '$lib/components/PlaylistsView.svelte';
	import PlaylistView from '$lib/components/PlaylistView.svelte';
	import QueuePanel from '$lib/components/QueuePanel.svelte';
	import SearchView from '$lib/components/SearchView.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import SignalPath from '$lib/components/SignalPath.svelte';
	import TrackList from '$lib/components/TrackList.svelte';
	import Transport from '$lib/components/Transport.svelte';
	import SettingsView from '$lib/components/settings/SettingsView.svelte';

	/** Seek step of the arrow keys (SPEC section 13). */
	const SEEK_STEP = 5;
	/** Volume step of Ctrl+Up/Down, in dB. */
	const VOLUME_STEP = 2;

	const fault = $derived(app.failure ?? themeFailure());
	const view = $derived(app.view);
	let searchField: HTMLInputElement | undefined = $state();

	// The hue follows the sound while tone colour is on (SPEC section 9.5).
	followTone();
	// The panels follow the width of the window (SPEC section 9.2).
	followWidth();

	function editing(target: EventTarget | null): boolean {
		return (
			target instanceof HTMLElement &&
			(target.isContentEditable || ['INPUT', 'SELECT', 'TEXTAREA'].includes(target.tagName))
		);
	}

	/** Default shortcuts (SPEC section 13). */
	function onKey(event: KeyboardEvent): void {
		if (!app.ready || app.firstRun) return;
		const ctrl = event.ctrlKey || event.metaKey;
		if (ctrl && event.key.toLowerCase() === 'f') {
			event.preventDefault();
			searchField?.focus();
			searchField?.select();
			return;
		}
		if (ctrl && event.key === ',') {
			event.preventDefault();
			navigate({ kind: 'settings', section: 'output' });
			return;
		}
		if (ctrl && event.key.toLowerCase() === 'm') {
			event.preventDefault();
			void setMiniPlayer(!app.mini);
			return;
		}
		if (ctrl && event.key.toLowerCase() === 'n') {
			event.preventDefault();
			navigate({ kind: app.view.kind === 'nowPlaying' ? 'tracks' : 'nowPlaying' });
			return;
		}
		if (event.key === 'Escape' && event.target === searchField) {
			setQuery('');
			searchField?.blur();
			return;
		}
		if (editing(event.target)) return;
		const hasTrack = player.snapshot?.track != null;
		if (event.key === ' ') {
			event.preventDefault();
			if (hasTrack) togglePlay().catch(() => {});
		} else if (ctrl && event.key === 'ArrowRight') {
			event.preventDefault();
			nextTrack().catch(() => {});
		} else if (ctrl && event.key === 'ArrowLeft') {
			event.preventDefault();
			previousTrack().catch(() => {});
		} else if (ctrl && (event.key === 'ArrowUp' || event.key === 'ArrowDown')) {
			event.preventDefault();
			const current = settings.value?.dsp.volumeDb ?? 0;
			const step = event.key === 'ArrowUp' ? VOLUME_STEP : -VOLUME_STEP;
			updateDsp({ volumeDb: Math.max(-60, Math.min(0, Math.max(current, -60) + step)) });
		} else if (!ctrl && (event.key === 'ArrowRight' || event.key === 'ArrowLeft') && hasTrack) {
			event.preventDefault();
			const step = event.key === 'ArrowRight' ? SEEK_STEP : -SEEK_STEP;
			seek(Math.max(0, player.position + step)).catch(() => {});
		}
	}
</script>

<svelte:window onkeydown={onKey} />

{#if fault}
	<main class="center"><p class="fault-text">{t(fault)}</p></main>
{:else if !app.ready}
	<main class="center"><p class="muted">{t('shell.loading')}</p></main>
{:else if app.firstRun}
	<FirstRun />
{:else if app.mini}
	<MiniPlayer />
{:else}
	<div
		class="shell"
		data-sidebar={layout.sidebar}
		class:with-queue={layout.queueDocked}
	>
		{#if layout.sidebar !== 'hidden'}
			<Sidebar mode={layout.sidebar} />
		{/if}
		<main class="content">
			<header class="head">
				{#if layout.sidebar === 'hidden'}
					<button
						type="button"
						class="icon-btn"
						aria-label={t('nav.library')}
						aria-expanded={layout.menuFloating}
						onclick={toggleMenu}><Icon name="menu" /></button
					>
				{/if}
				<label class="search">
					<Icon name="search" />
					<input
						bind:this={searchField}
						type="search"
						placeholder={t('search.placeholder')}
						aria-label={t('search.placeholder')}
						value={library.query}
						oninput={(event) => setQuery(event.currentTarget.value)}
					/>
					{#if library.query}
						<button
							type="button"
							class="clear"
							aria-label={t('search.clear')}
							onclick={() => setQuery('')}><Icon name="close" /></button
						>
					{/if}
				</label>
				{#if !layout.queueDocked}
					<button
						type="button"
						class="icon-btn"
						aria-label={t('queue.title')}
						aria-expanded={layout.queueFloating}
						onclick={toggleQueue}><Icon name="queue" /></button
					>
				{/if}
			</header>
			<div class="body">
				{#if library.query.trim()}
					<SearchView />
				{:else if view.kind === 'tracks'}
					<TrackList source={{ kind: 'library' }} view="tracks" />
				{:else if view.kind === 'albums'}
					<AlbumList />
				{:else if view.kind === 'album'}
					<AlbumView id={view.id} />
				{:else if view.kind === 'artists'}
					<BrowseView kind="artists" />
				{:else if view.kind === 'artist'}
					<BrowseView kind="artists" name={view.name} />
				{:else if view.kind === 'genres'}
					<BrowseView kind="genres" />
				{:else if view.kind === 'genre'}
					<BrowseView kind="genres" name={view.name} />
				{:else if view.kind === 'folders'}
					<BrowseView kind="folders" />
				{:else if view.kind === 'folder'}
					<BrowseView kind="folders" name={view.path} />
				{:else if view.kind === 'playlists'}
					<PlaylistsView />
				{:else if view.kind === 'playlist'}
					<PlaylistView id={view.id} />
				{:else if view.kind === 'nowPlaying'}
					<NowPlaying />
				{:else}
					<SettingsView section={view.section} />
				{/if}
			</div>
			{#if layout.queueFloating || layout.menuFloating}
				<!-- Tapping what is left of the content puts the panel away.
				     Only the content is covered: the transport stays within
				     reach while the listener reads the queue. -->
				<div class="scrim" role="presentation" onclick={closeOverlays}></div>
			{/if}
			{#if layout.queueFloating}
				<QueuePanel floating />
			{/if}
			{#if layout.menuFloating}
				<div class="drawer"><Sidebar mode="full" /></div>
			{/if}
		</main>
		{#if layout.queueDocked}
			<QueuePanel />
		{/if}
		<div class="transport"><Transport /></div>
		<div class="signal"><SignalPath /></div>
	</div>

	{#if carry.what && carry.at}
		<!-- What is being carried, following the pointer. It is a label, not
		     a control: pointer events pass straight through it. -->
		<div class="carried" style:left="{carry.at.x + 14}px" style:top="{carry.at.y + 12}px">
			{carry.what.count > 1
				? t('playlist.carried', { n: carry.what.count })
				: carry.what.label}
		</div>
	{/if}
{/if}

<style>
	.carried {
		position: fixed;
		z-index: 60;
		max-width: 260px;
		padding: 3px 9px;
		border: var(--onsa-hairline) solid var(--onsa-role-active);
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-raised);
		color: var(--onsa-text-primary);
		font-size: 12px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		pointer-events: none;
	}

	.center {
		display: grid;
		place-items: center;
		height: 100%;
	}

	.shell {
		position: relative;
		display: grid;
		grid-template-columns: 190px minmax(0, 1fr);
		grid-template-rows: minmax(0, 1fr) auto auto;
		grid-template-areas:
			'sidebar content'
			'transport transport'
			'signal signal';
		height: 100%;
		overflow: hidden;
	}

	.shell.with-queue {
		grid-template-columns: 190px minmax(0, 1fr) 290px;
		grid-template-areas:
			'sidebar content queue'
			'transport transport transport'
			'signal signal signal';
	}

	.shell[data-sidebar='icons'] {
		grid-template-columns: 52px minmax(0, 1fr);
	}

	.shell[data-sidebar='icons'].with-queue {
		grid-template-columns: 52px minmax(0, 1fr) 290px;
	}

	.shell[data-sidebar='hidden'] {
		grid-template-columns: minmax(0, 1fr);
		grid-template-areas:
			'content'
			'transport'
			'signal';
	}

	.shell > :global(.sidebar) {
		grid-area: sidebar;
	}

	/* Panels that cover the content rather than sit beside it. */
	.scrim {
		position: absolute;
		inset: 0;
		z-index: 8;
		background: rgb(0 0 0 / 0.35);
	}

	.drawer {
		position: absolute;
		top: 0;
		bottom: 0;
		left: 0;
		z-index: 9;
		width: min(190px, 70%);
		display: grid;
		box-shadow: 8px 0 24px rgb(0 0 0 / 0.45);
	}

	.content > :global(.queue.floating) {
		position: absolute;
		top: 0;
		right: 0;
		bottom: 0;
		z-index: 9;
		width: min(290px, 72%);
		box-shadow: -8px 0 24px rgb(0 0 0 / 0.45);
	}

	.content {
		position: relative;
		grid-area: content;
		display: grid;
		grid-template-rows: auto minmax(0, 1fr);
		min-width: 0;
		min-height: 0;
		overflow: hidden;
		background: var(--onsa-surface-app);
	}

	.shell > :global(.queue) {
		grid-area: queue;
	}

	.transport {
		grid-area: transport;
	}

	.signal {
		grid-area: signal;
	}

	.head {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 14px 8px;
	}

	.head .icon-btn {
		flex: none;
	}

	.search {
		flex: 1;
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
		max-width: 460px;
		padding: 6px 10px;
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		border-radius: var(--onsa-radius-md);
		background: var(--onsa-surface-well);
		color: var(--onsa-text-secondary);
	}

	.search:focus-within {
		border-color: var(--onsa-role-adjustable);
	}

	.search input {
		flex: 1;
		min-width: 0;
		border: 0;
		outline: none;
		background: none;
		color: var(--onsa-text-primary);
		font: inherit;
		font-size: 13px;
	}

	.search input::-webkit-search-cancel-button {
		display: none;
	}

	.clear {
		display: grid;
		place-items: center;
		padding: 0;
		border: 0;
		background: none;
		color: inherit;
		cursor: pointer;
	}

	.body {
		/* Pages ask this box how much room they have, not the window: the
		   sidebar and the queue take their share first (SPEC section 9.2). */
		container: content / inline-size;
		min-height: 0;
	}
</style>
