<!--
	Navigation (SPEC section 9.2): only what this build can already do.

	It shows its words where there is room, shrinks to a strip of icons when
	there is less, and steps aside into a drawer when there is less still
	(SPEC section 9.2). The entries never change; only how much of each one
	is drawn.
-->
<script lang="ts">
	import { app, navigate, type SettingsSection, type View } from '$lib/app.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';
	import { closeOverlays, layout } from '$lib/layout.svelte';
	import { library, scanSummary, setQuery } from '$lib/library.svelte';
	import { addToPlaylist, playlists } from '$lib/playlists.svelte';
	import { carry, dropTarget, type Carried } from '$lib/carry.svelte';
	import Icon from './Icon.svelte';

	interface Props {
		/** `icons` draws the strip; `full` draws the words as well. */
		mode?: 'full' | 'icons';
	}

	const { mode = 'full' }: Props = $props();

	type Entry = {
		view: View;
		label: MessageKey;
		icon: 'note' | 'disc' | 'artist' | 'genre' | 'folder' | 'playlist' | 'pencil' | 'download' | 'sliders' | 'eq' | 'shelf' | 'palette' | 'info';
		/** Views that count as this entry being the one in use. */
		kinds: View['kind'][];
		section?: SettingsSection;
	};

	const libraryPages: Entry[] = [
		{ view: { kind: 'tracks' }, label: 'nav.tracks', icon: 'note', kinds: ['tracks'] },
		{ view: { kind: 'albums' }, label: 'nav.albums', icon: 'disc', kinds: ['albums', 'album'] },
		{ view: { kind: 'artists' }, label: 'nav.artists', icon: 'artist', kinds: ['artists', 'artist'] },
		{ view: { kind: 'genres' }, label: 'nav.genres', icon: 'genre', kinds: ['genres', 'genre'] },
		{ view: { kind: 'folders' }, label: 'nav.folders', icon: 'folder', kinds: ['folders', 'folder'] },
		{
			view: { kind: 'playlists' },
			label: 'nav.playlists',
			icon: 'playlist',
			kinds: ['playlists', 'playlist']
		},
		{ view: { kind: 'tidy' }, label: 'nav.tidy', icon: 'pencil', kinds: ['tidy'] },
		{
			view: { kind: 'downloads' },
			label: 'nav.downloads',
			icon: 'download',
			kinds: ['downloads']
		}
	];

	const settingsPages: Entry[] = (
		[
			['output', 'settings.output', 'sliders'],
			['dsp', 'settings.dsp', 'eq'],
			['library', 'settings.library', 'shelf'],
			['metadata', 'settings.metadata', 'info'],
			['lyrics', 'settings.lyrics', 'info'],
			['appearance', 'settings.appearance', 'palette'],
			['about', 'settings.about', 'info']
		] as const
	).map(([section, label, icon]) => ({
		view: { kind: 'settings', section } as View,
		label,
		icon,
		kinds: ['settings'],
		section
	}));

	const view = $derived(app.view);

	/** How many playlists the sidebar lists before it says "see them all". */
	const SHORTLIST = 6;

	// The ones changed most recently, which is the order the backend lists
	// them in; the rest are one click away on the playlist page.
	const recent = $derived(playlists.all.slice(0, SHORTLIST));
	const more = $derived(playlists.all.length > recent.length);

	/** What was carried has landed on a playlist. */
	function accept(id: number, what: Carried): void {
		addToPlaylist(id, what.context).catch(() => {});
	}

	function current(entry: Entry): boolean {
		if (library.query) return false;
		if (!entry.kinds.includes(view.kind)) return false;
		if (entry.section) return view.kind === 'settings' && view.section === entry.section;
		return true;
	}

	function go(entry: Entry): void {
		setQuery('');
		navigate(entry.view);
		// In the drawer, choosing somewhere to go is also done with it.
		closeOverlays();
	}
</script>

<nav class="sidebar" data-mode={mode} aria-label={t('nav.library')}>
	{#if mode === 'full'}
		<div class="mark">Onsa</div>
	{/if}

	{#snippet group(heading: MessageKey, entries: Entry[])}
		{#if mode === 'full'}
			<h2 class="label">{t(heading)}</h2>
		{:else}
			<div class="rule" role="presentation"></div>
		{/if}
		{#each entries as entry (entry.label)}
			<button
				type="button"
				class="item"
				aria-current={current(entry)}
				title={t(entry.label)}
				aria-label={t(entry.label)}
				onclick={() => go(entry)}
			>
				<Icon name={entry.icon} />
				{#if mode === 'full'}<span class="text ellipsis">{t(entry.label)}</span>{/if}
			</button>
		{/each}
	{/snippet}

	{@render group('nav.library', libraryPages)}

	{#if mode === 'full' && playlists.all.length > 0}
		<h2 class="label">{t('nav.playlists')}</h2>
		{#each recent as playlist (playlist.id)}
			<button
				type="button"
				class="item playlist"
				class:over={carry.over === `playlist:${playlist.id}`}
				aria-current={view.kind === 'playlist' && view.id === playlist.id}
				title={playlist.name}
				use:dropTarget={{
					key: `playlist:${playlist.id}`,
					accept: (what) => accept(playlist.id, what)
				}}
				onclick={() => {
					setQuery('');
					navigate({ kind: 'playlist', id: playlist.id });
					closeOverlays();
				}}
			>
				<Icon name={playlist.kind === 'smart' ? 'eq' : 'playlist'} />
				<span class="text ellipsis">{playlist.name}</span>
			</button>
		{/each}
		{#if more}
			<button
				type="button"
				class="item all"
				onclick={() => {
					setQuery('');
					navigate({ kind: 'playlists' });
					closeOverlays();
				}}
			>
				<span class="text ellipsis">{t('nav.allPlaylists')}</span>
			</button>
		{/if}
	{/if}

	{@render group('nav.settings', settingsPages)}

	{#if library.scan.running}
		{#if mode === 'full'}
			<p class="scan numeric ellipsis">{scanSummary()}</p>
		{:else}
			<p class="scan dot" title={scanSummary()} aria-label={scanSummary()}>●</p>
		{/if}
	{/if}
</nav>

<style>
	.sidebar {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 16px 10px;
		min-height: 0;
		overflow-y: auto;
		overflow-x: hidden;
		border-right: var(--onsa-hairline) solid var(--onsa-surface-line);
		background: var(--onsa-surface-body);
		scrollbar-width: thin;
	}

	.sidebar[data-mode='icons'] {
		align-items: center;
		padding: 12px 6px;
	}

	.mark {
		padding: 0 8px 12px;
		font-size: 20px;
		color: var(--onsa-text-primary);
	}

	h2 {
		margin: 14px 8px 4px;
		font-size: 10.5px;
		font-weight: 500;
	}

	.rule {
		width: 20px;
		height: var(--onsa-hairline);
		margin: 10px 0;
		background: var(--onsa-surface-line);
	}

	.item {
		display: flex;
		align-items: center;
		gap: 9px;
		min-width: 0;
		padding: 6px 8px;
		border: 0;
		border-radius: var(--onsa-radius-sm);
		background: none;
		color: var(--onsa-text-secondary);
		font: inherit;
		font-size: 13px;
		text-align: left;
		cursor: pointer;
	}

	.item :global(svg) {
		flex: none;
		width: 17px;
		height: 17px;
		fill: currentColor;
	}

	.sidebar[data-mode='icons'] .item {
		padding: 8px;
	}

	.item:hover {
		color: var(--onsa-text-primary);
		background: var(--onsa-surface-raised);
	}

	.item[aria-current='true'] {
		color: var(--onsa-role-active);
		background: var(--onsa-surface-raised);
	}

	/* A playlist the carried track is over, waiting to take it. */
	.item.over {
		color: var(--onsa-role-active);
		box-shadow: inset 0 0 0 var(--onsa-hairline) var(--onsa-role-active);
	}

	.item.playlist :global(svg) {
		width: 15px;
		height: 15px;
	}

	.item.all {
		padding-left: 34px;
		font-size: 12px;
		color: var(--onsa-text-secondary);
	}

	.scan {
		margin: auto 8px 0;
		padding-top: 12px;
		font-size: 11px;
		color: var(--onsa-role-caution);
	}

	.scan.dot {
		margin: auto 0 0;
		font-size: 10px;
	}
</style>
