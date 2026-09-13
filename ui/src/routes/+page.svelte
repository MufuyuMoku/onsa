<!--
	The window (SPEC section 9.2): sidebar, main area, queue, transport and
	the signal path strip. The first screen shows until the library has a
	folder.
-->
<script lang="ts">
	import { nextTrack, previousTrack, seek, togglePlay } from '$lib/backend';
	import { followTone } from '$lib/analysis.svelte';
	import { app, navigate, setMiniPlayer } from '$lib/app.svelte';
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
	<div class="shell">
		<Sidebar />
		<main class="content">
			<header class="head">
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
			</header>
			<div class="body">
				{#if library.query.trim()}
					<SearchView />
				{:else if view.kind === 'tracks'}
					<TrackList source={{ kind: 'library' }} />
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
				{:else if view.kind === 'nowPlaying'}
					<NowPlaying />
				{:else}
					<SettingsView section={view.section} />
				{/if}
			</div>
		</main>
		<QueuePanel />
		<div class="transport"><Transport /></div>
		<div class="signal"><SignalPath /></div>
	</div>
{/if}

<style>
	.center {
		display: grid;
		place-items: center;
		height: 100%;
	}

	.shell {
		display: grid;
		grid-template-columns: 190px minmax(0, 1fr) 290px;
		grid-template-rows: minmax(0, 1fr) auto auto;
		grid-template-areas:
			'sidebar content queue'
			'transport transport transport'
			'signal signal signal';
		height: 100%;
	}

	.shell > :global(.sidebar) {
		grid-area: sidebar;
	}

	.content {
		grid-area: content;
		display: grid;
		grid-template-rows: auto minmax(0, 1fr);
		min-width: 0;
		min-height: 0;
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
		padding: 12px 14px 8px;
	}

	.search {
		display: flex;
		align-items: center;
		gap: 8px;
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
		min-height: 0;
	}
</style>
