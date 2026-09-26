<!--
	A track list: the whole library (sorted, paged from the backend) or a
	given list (an album, search results). Double-click or Enter plays from
	that row, with the rest of the list as the queue.
-->
<script lang="ts">
	import {
		trackCount,
		trackPlace,
		tracksPage,
		type PlayContext,
		type QueuePlace,
		type Track
	} from '$lib/backend';
	import {
		COLUMNS,
		column,
		shownColumns,
		spreadWidths,
		template,
		titleWidth,
		widthOf,
		type ColumnKey
	} from '$lib/columns';
	import { prefsFor, resetColumns, setShown, setWidth, type ListView } from '$lib/columns.svelte';
	import { clock, fileName } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import { measure } from '$lib/layout.svelte';
	import { library, sortBy } from '$lib/library.svelte';
	import {
		askedToShowPlaying,
		placeApplies,
		placeFor,
		rememberPlace
	} from '$lib/places.svelte';
	import { settings } from '$lib/settings.svelte';
	import { untrack } from 'svelte';
	import { addToQueue, player, playContext } from '$lib/player.svelte';
	import { addToPlaylist, playlists } from '$lib/playlists.svelte';
	import { startCarry } from '$lib/carry.svelte';
	import Icon from './Icon.svelte';
	import TrackEditor from './TrackEditor.svelte';
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

	/**
	 * A list with an order of its own — an album, a playlist — cannot be
	 * re-sorted, so its position does not belong to any ordering.
	 */
	const fixedOrder = untrack(() => source.kind !== 'library');

	function moved(top: number): void {
		rememberPlace(view, {
			top,
			sort: fixedOrder ? 'fixed' : library.sort,
			descending: fixedOrder ? false : library.descending
		});
	}

	/** The list itself, for scrolling it from outside the rows. */
	let list = $state<ReturnType<typeof VirtualList> | undefined>();
	/** A row to point at for a moment, after jumping to it. */
	let marked = $state<number | null>(null);
	let marking: ReturnType<typeof setTimeout> | null = null;

	/** How long the row stays pointed at. Long enough to find, not to sit. */
	const MARKED_FOR = 2200;

	/**
	 * Which row the playing track is on.
	 *
	 * The library list is paged, so the interface has no list to search:
	 * only the database knows, and it is asked in the ordering on screen.
	 * A list that carries its own tracks already holds the answer.
	 */
	async function playingRow(): Promise<number | null> {
		const id = player.snapshot?.track?.id;
		if (id === undefined || id === null || id < 0) return null;
		if (source.kind === 'library') {
			const at = await trackPlace(library.sort, library.descending, id).catch(() => null);
			return at ?? null;
		}
		const at = source.tracks.findIndex((track) => track.id === id);
		return at >= 0 ? at : null;
	}

	/** Scrolls to the playing track, and points at it when asked to. */
	async function goToPlaying(point: boolean): Promise<void> {
		const at = await playingRow();
		if (at === null) return;
		list?.reveal(at);
		if (!point) return;
		marked = at;
		if (marking) clearTimeout(marking);
		marking = setTimeout(() => (marked = null), MARKED_FOR);
	}

	// Asked from the window: the button up there and the shortcut both come
	// through here, because the list that can scroll is this one.
	let answered = askedToShowPlaying();
	$effect(() => {
		const asked = askedToShowPlaying();
		if (asked === answered) return;
		answered = asked;
		void goToPlaying(true);
	});

	/**
	 * Opening the list where it was left.
	 *
	 * Three things have to be true before this can be answered, and none of
	 * them is true when the list is built: the settings have to have arrived
	 * (when Onsa has just started they have not), the list has to have a
	 * length, and the viewport has to be tall enough to be scrolled that far
	 * — a browser clamps an offset past the end back to the top. So it is
	 * tried until it takes, and then never again: after that the listener's
	 * own scrolling is the truth.
	 *
	 * A position only means anything under the ordering it was taken in:
	 * row four hundred by title is a different song from row four hundred by
	 * year. A position from another ordering is left alone, and the list
	 * opens on whatever is playing instead — the one row somebody is likely
	 * to be looking for.
	 */
	let placed = false;
	$effect(() => {
		if (placed || !settings.value || count === 0) return;
		const stored = placeFor(view);
		if (
			stored &&
			placeApplies(stored, {
				fixed: fixedOrder,
				sort: library.sort,
				descending: library.descending
			})
		) {
			placed = list?.scrollTo(stored.top) ?? false;
			return;
		}
		placed = true;
		void goToPlaying(false);
	});

	// A list reordered under the listener's feet has no business staying
	// where it was: row four hundred is a different song now.
	let ordering = `${library.sort}:${library.descending}`;
	$effect(() => {
		const now = `${library.sort}:${library.descending}`;
		if (fixedOrder || now === ordering) return;
		ordering = now;
		marked = null;
		list?.scrollTo(0);
	});

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

	/** The song whose tags are being edited, if any (SPEC section 8). */
	let editing = $state<number | null>(null);

	function editOne(): void {
		const chosen = menu;
		menu = null;
		if (chosen) editing = chosen.id;
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
		// The grip keeps the pointer for the whole drag, so a hand that
		// moves faster than the redraw — or leaves the window altogether —
		// is still the hand holding this divider.
		const grip = event.currentTarget as HTMLElement | null;
		grip?.setPointerCapture?.(event.pointerId);
		sizing = { key, startX: event.clientX, startWidth: drawnWidth(key) };
	}

	function onSizing(event: PointerEvent): void {
		if (!sizing) return;
		const asked = sizing.startWidth + (event.clientX - sizing.startX);
		live = spreadWidths(width, shown, prefs, sizing.key, asked);
	}

	function endSizing(): void {
		const done = sizing;
		sizing = null;
		if (!done) return;
		const settled = live;
		live = {};
		// Only what it was let go at is worth storing, and only what
		// actually moved: the column dragged, and any that gave way to it.
		for (const [key, wanted] of Object.entries(settled)) {
			if (typeof wanted === 'number') setWidth(view, key as ColumnKey, wanted);
		}
	}

	/**
	 * The width a column is actually drawn at right now.
	 *
	 * A drag starts from what is on screen, not from what is stored: the
	 * title may be narrower than its stored width when the window is too
	 * small for it, and starting from the stored number would make the
	 * first pixel of the drag jump.
	 */
	function drawnWidth(key: ColumnKey): number {
		if (key !== 'title') return widthOf(key, prefs);
		return titleWidth(width, shown, prefs);
	}

	/**
	 * Double-click on a divider: the column takes the width of what is in
	 * it, the way a file list does.
	 *
	 * What it measures is the rows that are drawn, which in a list of a
	 * hundred thousand songs is the only honest answer — the others have
	 * never been laid out and have no width to ask for.
	 */
	function fitToContents(key: ColumnKey): void {
		const at = shown.indexOf(key);
		if (at < 0) return;
		// One past the number column at the front.
		const nth = at + 2;
		const cells = document.querySelectorAll<HTMLElement>(
			`.tracklist .body .row > *:nth-child(${nth})`
		);
		let found = 0;
		for (const cell of cells) found = Math.max(found, cell.scrollWidth);
		const head = document.querySelector<HTMLElement>(
			`.tracklist .head .cell:nth-of-type(${at + 1}) .sort, .tracklist .head .cell:nth-of-type(${at + 1}) > .ellipsis`
		);
		if (head) found = Math.max(found, head.scrollWidth + 18);
		if (found <= 0) return;
		// A little air, so the longest line does not sit against the next
		// column's edge — and never wider than there is room for.
		const wanted = Math.max(column(key).min, Math.round(found + 12));
		for (const [one, wide] of Object.entries(spreadWidths(width, shown, prefs, key, wanted))) {
			if (typeof wide === 'number') setWidth(view, one as ColumnKey, wide);
		}
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
						onpointermove={onSizing}
						onpointerup={endSizing}
						onpointercancel={endSizing}
						ondblclick={() => fitToContents(key)}
						onkeydown={(event) => sizeByKey(event, key)}
					></button>
				{/if}
			</span>
		{/each}
		<span class="menu-cell">
			<button
				type="button"
				class="chooser"
				aria-label={t('column.toPlaying')}
				title={t('column.toPlayingHint')}
				disabled={!player.snapshot?.track}
				onclick={() => goToPlaying(true)}><Icon name="target" /></button
			>
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
			<VirtualList
				bind:this={list}
				{count}
				rowHeight={ROW}
				{load}
				version={library.version}
				label={t('nav.tracks')}
				onmoved={moved}
			>
				{#snippet row(track: Track | undefined, index: number)}
					{#if track}
						<div
							class="row listrow"
							class:alt={index % 2 === 1}
							class:marked={marked === index}
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
		<li>
			<button type="button" onclick={() => editOne()}>{t('editor.open')}</button>
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

{#if editing !== null}
	<TrackEditor trackId={editing} onclose={() => (editing = null)} />
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

	/* The strip at the right end: the two buttons in the head, empty in
	   every row, so the head and the rows keep the same grid. */
	.menu-cell {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 2px;
	}

	.chooser:disabled {
		opacity: 0.4;
		cursor: default;
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

	/* Just jumped to: pointed at for a moment, so the eye can find it
	   among rows that all look alike. */
	.row.marked {
		outline: 1px solid var(--onsa-role-adjustable);
		outline-offset: -1px;
		background: var(--onsa-surface-raised);
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
