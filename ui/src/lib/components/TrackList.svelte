<!--
	A track list: the whole library (sorted, paged from the backend) or a
	given list (an album, search results). Double-click or Enter plays from
	that row, with the rest of the list as the queue.
-->
<script lang="ts">
	import {
		trackCount,
		tracksPage,
		type PlayContext,
		type QueuePlace,
		type Track
	} from '$lib/backend';
	import {
		COLUMNS,
		column,
		shownColumns,
		template,
		widthOf,
		type ColumnKey
	} from '$lib/columns';
	import { prefsFor, resetColumns, setShown, setWidth, type ListView } from '$lib/columns.svelte';
	import { clock } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import { measure } from '$lib/layout.svelte';
	import { library, sortBy } from '$lib/library.svelte';
	import { addToQueue, player, playContext } from '$lib/player.svelte';
	import { addToPlaylist, playlists } from '$lib/playlists.svelte';
	import { startCarry } from '$lib/carry.svelte';
	import Icon from './Icon.svelte';
	import VirtualList from './VirtualList.svelte';

	type Source =
		| { kind: 'library' }
		| { kind: 'album'; albumId: number; tracks: Track[] }
		| { kind: 'tracks'; tracks: Track[] };

	interface Props {
		source: Source;
		/** Which list this is, so its column widths are its own. */
		view: ListView;
		showAlbum?: boolean;
		showNumber?: boolean;
	}

	let { source, view, showAlbum = true, showNumber = false }: Props = $props();

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

	function contextFor(index: number): PlayContext {
		if (source.kind === 'library') {
			return { kind: 'library', sort: library.sort, descending: library.descending, index };
		}
		if (source.kind === 'album') {
			return { kind: 'album', albumId: source.albumId, index };
		}
		return { kind: 'tracks', ids: source.tracks.map((track) => track.id), index };
	}

	function activate(index: number): void {
		playContext(contextFor(index)).catch(() => {});
	}

	/** The right-click menu: one track, added where the listener asks. */
	let menu = $state<{ x: number; y: number; id: number } | null>(null);

	function openMenu(event: MouseEvent, track: Track): void {
		event.preventDefault();
		if (track.id < 0) return;
		menu = { x: event.clientX, y: event.clientY, id: track.id };
	}

	function addOne(place: QueuePlace): void {
		const chosen = menu;
		menu = null;
		if (!chosen) return;
		addToQueue({ kind: 'tracks', ids: [chosen.id], index: 0 }, place).catch(() => {});
	}

	/** The same track, into a playlist the listener already has. */
	function addOneTo(playlistId: number): void {
		const chosen = menu;
		menu = null;
		if (!chosen) return;
		addToPlaylist(playlistId, { kind: 'tracks', ids: [chosen.id], index: 0 }).catch(() => {});
	}

	/** Carrying a row out of the list, onto a playlist in the sidebar. */
	function lift(event: PointerEvent, track: Track): void {
		if (track.id < 0) return;
		startCarry(event, () => ({
			// The one row that was picked up, not the list it came from.
			context: { kind: 'tracks', ids: [track.id], index: 0 },
			label: track.title ?? fileName(track.path),
			count: 1
		}));
	}

	// Only manual playlists take tracks: a smart one is its rules.
	const manual = $derived(playlists.all.filter((playlist) => playlist.kind === 'manual'));

	function fileName(path: string): string {
		return path.split(/[\\/]/).pop() ?? path;
	}

	// Title and length always stay; the rest go in this order as the list
	// narrows — the year first, then the album, then the artist. A column is
	// dropped rather than squeezed: squeezed columns end up on top of each
	// other (SPEC section 9.2). The widths themselves are the listener's,
	// and are remembered per list.
	const prefs = $derived(prefsFor(view));

	/** Width of the list itself, which is what the columns have to fit in. */
	let width = $state(1000);

	const shown = $derived(
		shownColumns(width, prefs, (key) => showAlbum || key !== 'album')
	);
	const has = $derived({
		artist: shown.includes('artist'),
		album: shown.includes('album'),
		year: shown.includes('year')
	});


	/** The column being dragged wider or narrower, while it is happening. */
	let sizing = $state<{ key: ColumnKey; startX: number; startWidth: number } | null>(null);
	/** The width under the pointer, before it is written down. */
	let live = $state<Partial<Record<ColumnKey, number>>>({});

	function startSizing(event: PointerEvent, key: ColumnKey): void {
		if (event.button !== 0) return;
		event.preventDefault();
		event.stopPropagation();
		sizing = { key, startX: event.clientX, startWidth: widthOf(key, prefs) };
		window.addEventListener('pointermove', onSizing);
		window.addEventListener('pointerup', endSizing);
		window.addEventListener('pointercancel', endSizing);
	}

	function onSizing(event: PointerEvent): void {
		if (!sizing) return;
		const wanted = sizing.startWidth + (event.clientX - sizing.startX);
		live = { ...live, [sizing.key]: Math.max(column(sizing.key).min, Math.round(wanted)) };
	}

	function endSizing(): void {
		window.removeEventListener('pointermove', onSizing);
		window.removeEventListener('pointerup', endSizing);
		window.removeEventListener('pointercancel', endSizing);
		const done = sizing;
		sizing = null;
		if (!done) return;
		const wanted = live[done.key];
		live = {};
		// Only the width it was let go at is worth storing.
		if (typeof wanted === 'number' && wanted !== done.startWidth) setWidth(view, done.key, wanted);
	}

	/** A divider moved by the keyboard, for those who do not drag. */
	function sizeByKey(event: KeyboardEvent, key: ColumnKey): void {
		const step = event.key === 'ArrowLeft' ? -16 : event.key === 'ArrowRight' ? 16 : 0;
		if (step === 0) return;
		event.preventDefault();
		setWidth(view, key, Math.max(column(key).min, widthOf(key, prefs) + step));
	}

	/** What the list draws right now: a drag in progress wins over the store. */
	const drawn = $derived.by(() => {
		const merged = { ...prefs, widths: { ...prefs.widths, ...live } };
		return { template: template(width, shown, merged), prefs: merged };
	});

	/** The column menu: which columns are on the list, and a way back. */
	let chooser = $state(false);
	const optional = COLUMNS.filter((entry) => entry.optional);

</script>

<div class="tracklist" use:measure={(seen) => (width = seen)} style:--columns={drawn.template}>
	<div class="head label" class:sizing={sizing !== null} role="row">
		<span class="num">{t('column.number')}</span>
		{#each shown as key (key)}
			{@const entry = column(key)}
			<span class="cell" class:right={key === 'duration'}>
				{#if sortable}
					<button
						type="button"
						class="sort"
						aria-pressed={library.sort === key}
						title={t('column.sortHint')}
						onclick={() => sortBy(key)}
					>
						<span class="ellipsis">{t(entry.label)}</span>
						{#if library.sort === key}<span aria-hidden="true"
								>{library.descending ? '▼' : '▲'}</span
							>{/if}
					</button>
				{:else}
					<span class="ellipsis">{t(entry.label)}</span>
				{/if}
				{#if entry.sizable}
					<!-- The divider between this column and the next one. -->
					<button
						type="button"
						class="grip"
						class:held={sizing?.key === key}
						aria-label={t('column.resize', { name: t(entry.label) })}
						title={t('column.resizeHint')}
						onpointerdown={(event) => startSizing(event, key)}
						onkeydown={(event) => sizeByKey(event, key)}
					></button>
				{/if}
			</span>
		{/each}
		<span class="menu-cell">
			<button
				type="button"
				class="chooser"
				aria-label={t('column.choose')}
				title={t('column.choose')}
				aria-expanded={chooser}
				onclick={() => (chooser = !chooser)}><Icon name="more" /></button
			>
		</span>
		{#if chooser}
			<!-- Clicking anywhere else puts the menu away. -->
			<div class="backdrop" role="presentation" onclick={() => (chooser = false)}></div>
			<menu class="menu columns">
				{#each optional as entry (entry.key)}
					<li>
						<label>
							<input
								type="checkbox"
								checked={!prefs.hidden.includes(entry.key)}
								disabled={entry.key === 'album' && !showAlbum}
								onchange={(event) => setShown(view, entry.key, event.currentTarget.checked)}
							/>
							<span>{t(entry.label)}</span>
						</label>
					</li>
				{/each}
				<li>
					<button
						type="button"
						onclick={() => {
							chooser = false;
							resetColumns(view);
						}}>{t('column.reset')}</button
					>
				</li>
			</menu>
		{/if}
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
							onpointerdown={(event) => lift(event, track)}
							oncontextmenu={(event) => openMenu(event, track)}
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
							{#if has.artist}
								<span class="ellipsis muted">{track.artist ?? t('track.unknownArtist')}</span>
							{/if}
							{#if has.album}<span class="ellipsis muted">{track.album ?? ''}</span>{/if}
							{#if has.year}<span class="numeric muted">{track.year ?? ''}</span>{/if}
							<span class="numeric muted right"
								>{clock(track.durationMs === null ? null : track.durationMs / 1000)}</span
							>
							<span class="menu-cell"></span>
						</div>
					{:else}
						<div class="row placeholder"></div>
					{/if}
				{/snippet}
			</VirtualList>
		{/if}
	</div>
</div>

{#if menu}
	<!-- Closing the menu is what the backdrop is for; it is not a control. -->
	<div
		class="backdrop"
		role="presentation"
		onclick={() => (menu = null)}
		oncontextmenu={(event) => {
			event.preventDefault();
			menu = null;
		}}
	></div>
	<menu class="menu" style:left="{menu.x}px" style:top="{menu.y}px">
		<li>
			<button type="button" onclick={() => addOne('next')}>{t('queue.playNext')}</button>
		</li>
		<li>
			<button type="button" onclick={() => addOne('end')}>{t('queue.addToEnd')}</button>
		</li>
		{#if manual.length > 0}
			<li class="heading label" role="presentation">{t('playlist.addTo')}</li>
			{#each manual as playlist (playlist.id)}
				<li>
					<button type="button" class="ellipsis" onclick={() => addOneTo(playlist.id)}
						>{playlist.name}</button
					>
				</li>
			{/each}
		{/if}
	</menu>
{/if}

<svelte:window
	onkeydown={(event) => {
		if (event.key === 'Escape') menu = null;
	}}
/>

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		z-index: 10;
	}

	.menu {
		position: fixed;
		z-index: 11;
		display: grid;
		margin: 0;
		padding: 4px;
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-raised);
		box-shadow: 0 10px 24px rgb(0 0 0 / 0.45);
		list-style: none;
	}

	.menu {
		max-height: min(60vh, 420px);
		max-width: 260px;
		overflow-y: auto;
		scrollbar-width: thin;
	}

	.menu .heading {
		margin-top: 4px;
		padding: 6px 10px 2px;
		border-top: var(--onsa-hairline) solid var(--onsa-surface-line);
		font-size: 10px;
	}

	.menu button {
		width: 100%;
		padding: 6px 14px 6px 10px;
		border: 0;
		border-radius: var(--onsa-radius-sm);
		background: none;
		color: var(--onsa-text-primary);
		font: inherit;
		font-size: 12.5px;
		text-align: left;
		white-space: nowrap;
		cursor: pointer;
	}

	.menu button:hover {
		background: var(--onsa-surface-body);
		color: var(--onsa-role-active);
	}

	.tracklist {
		display: grid;
		grid-template-rows: auto 1fr;
		height: 100%;
		min-height: 0;
	}

	.head,
	.row {
		display: grid;
		grid-template-columns: var(--columns);
		align-items: center;
		gap: 10px;
		padding: 0 14px;
	}

	/* Nothing in a row may grow past its column, headings included. */
	.head > *,
	.row > * {
		min-width: 0;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}

	.head {
		position: relative;
		height: 30px;
		font-size: 11px;
		border-bottom: var(--onsa-hairline) solid var(--onsa-surface-line);
	}

	/* While a divider is being dragged, nothing else takes the pointer. */
	.head.sizing {
		cursor: col-resize;
		user-select: none;
	}

	.cell {
		position: relative;
		display: flex;
		align-items: center;
		gap: 4px;
		min-width: 0;
		/* The divider reaches into the gap beside the column, so this cell
		   must not clip it; the heading inside does its own trimming. */
		overflow: visible;
	}

	.cell.right {
		justify-content: flex-end;
	}

	.sort {
		display: flex;
		align-items: center;
		gap: 4px;
		min-width: 0;
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

	/* The divider between two columns: a hairline with a wider place to
	   take hold of it, so it can be caught without careful aiming. */
	.grip {
		position: absolute;
		padding: 0;
		border: 0;
		background: none;
		top: -4px;
		bottom: -4px;
		right: -7px;
		width: 14px;
		/* Above the next column: the grip reaches into it, and without this
		   that column would take the press meant for the divider. */
		z-index: 3;
		cursor: col-resize;
		touch-action: none;
	}

	.grip::after {
		content: '';
		position: absolute;
		top: 6px;
		bottom: 6px;
		left: 7px;
		width: var(--onsa-hairline);
		background: var(--onsa-surface-line);
	}

	.grip:hover::after,
	.grip:focus-visible::after,
	.grip.held::after {
		top: 0;
		bottom: 0;
		background: var(--onsa-role-adjustable);
	}

	.grip:focus-visible {
		outline: none;
	}

	/* The strip at the right end: the menu in the head, empty in every row,
	   so the head and the rows keep the same grid. */
	.menu-cell {
		display: grid;
		place-items: center;
	}

	.chooser {
		display: grid;
		place-items: center;
		width: 20px;
		height: 20px;
		padding: 0;
		border: 0;
		border-radius: var(--onsa-radius-sm);
		background: none;
		color: var(--onsa-text-secondary);
		cursor: pointer;
	}

	.chooser:hover {
		color: var(--onsa-text-primary);
		background: var(--onsa-surface-raised);
	}

	.chooser :global(svg) {
		width: 14px;
		height: 14px;
	}

	.menu.columns {
		position: absolute;
		top: calc(100% + 2px);
		right: 0;
		left: auto;
	}

	.menu.columns label {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 5px 14px 5px 10px;
		font-size: 13px;
		letter-spacing: normal;
		text-transform: none;
		color: var(--onsa-text-primary);
		white-space: nowrap;
		cursor: pointer;
	}

	.menu.columns li:last-child {
		margin-top: 4px;
		border-top: var(--onsa-hairline) solid var(--onsa-surface-line);
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
