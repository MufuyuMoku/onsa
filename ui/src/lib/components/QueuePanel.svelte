<!--
	The queue (SPEC section 6.1): what plays next, in the order it will play.
	Double-click jumps, drag reorders, and shuffle keeps the listed order to
	go back to.
-->
<script lang="ts">
	import { jump, type QueueEntry } from '$lib/backend';
	import { clock } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import {
		cycleRepeat,
		emptyQueue,
		moveQueueEntry,
		player,
		removeQueueEntry,
		toggleShuffle
	} from '$lib/player.svelte';
	import { closeOverlays } from '$lib/layout.svelte';
	import { reorderable, reorderKey } from '$lib/reorder';
	import { settings } from '$lib/settings.svelte';
	import { saveQueueAsPlaylist } from '$lib/playlists.svelte';
	import Icon from './Icon.svelte';
	import NamePrompt from './NamePrompt.svelte';
	import VirtualList from './VirtualList.svelte';

	interface Props {
		/** Whether the panel is covering the content rather than beside it. */
		floating?: boolean;
	}

	const { floating = false }: Props = $props();

	const ROW = 44;

	const items = $derived(player.queue.items);
	// Which entry plays is the engine's answer, carried in the snapshot; the
	// position only says where to scroll to (SPEC section 6.1).
	const current = $derived(player.snapshot?.currentId ?? null);
	const at = $derived(player.snapshot?.current ?? null);
	const shuffle = $derived(player.snapshot?.shuffle ?? false);
	const repeat = $derived(settings.value?.playback.repeat ?? 'off');
	const repeatLabel = $derived(
		repeat === 'one' ? 'queue.repeatOne' : repeat === 'all' ? 'queue.repeatAll' : 'queue.repeatOff'
	);

	const load = $derived.by(() => {
		const list = items;
		return (offset: number, limit: number) => Promise.resolve(list.slice(offset, offset + limit));
	});

	let list: VirtualList<QueueEntry> | undefined = $state();

	$effect(() => {
		if (at !== null) list?.reveal(at);
	});

	function fileName(path: string): string {
		return path.split(/[\\/]/).pop() ?? path;
	}

	// Saving the queue is how a session of clicking about becomes a playlist
	// worth keeping (SPEC section 6.2).
	let saving = $state(false);

	async function saveQueue(name: string): Promise<void> {
		saving = false;
		try {
			await saveQueueAsPlaylist(name);
		} catch {
			// A playlist that cannot be written leaves the queue as it is.
		}
	}

	/** A row landed somewhere else; the engine is told which entry moved. */
	function reorder(from: number, to: number): void {
		const entry = items[from];
		if (!entry) return;
		moveQueueEntry(entry.entryId, to).catch(() => {});
	}
</script>

<aside class="queue" class:floating>
	<header class="head">
		<h2 class="label">{t('queue.title')}</h2>
		<span class="grow"></span>
		{#if items.length > 0}
			<span class="numeric muted">{t('queue.count', { n: items.length })}</span>
		{/if}
		{#if floating}
			<button type="button" class="icon-btn" aria-label={t('search.clear')} onclick={closeOverlays}>
				<Icon name="close" />
			</button>
		{/if}
	</header>

	<div class="tools">
		<button
			type="button"
			class="chip"
			aria-pressed={shuffle}
			title={t('queue.shuffle')}
			disabled={items.length === 0}
			onclick={() => toggleShuffle().catch(() => {})}
		>
			{t('queue.shuffle')}
		</button>
		<button
			type="button"
			class="chip"
			aria-pressed={repeat !== 'off'}
			title={t(repeatLabel)}
			onclick={() => cycleRepeat().catch(() => {})}
		>
			{t('queue.repeat')}{repeat === 'one' ? ' 1' : ''}
		</button>
		<button
			type="button"
			class="chip"
			disabled={items.length === 0}
			onclick={() => emptyQueue().catch(() => {})}
		>
			{t('queue.clear')}
		</button>
		<button
			type="button"
			class="chip"
			title={t('playlist.saveQueue')}
			disabled={items.length === 0}
			onclick={() => (saving = true)}
		>
			<Icon name="playlist" />
		</button>
	</div>

	<div class="body">
		{#if items.length === 0}
			<p class="muted empty">{t('queue.empty')}</p>
		{:else}
			<!-- Pointer events, not HTML5 drag and drop: Tauri's file-drop
			     target owns the native drag loop on Windows. -->
			<div
				class="reorder"
				use:reorderable={{ count: () => items.length, move: reorder, row: '.item' }}
			>
			<VirtualList
				bind:this={list}
				count={items.length}
				rowHeight={ROW}
				{load}
				version={items}
				label={t('queue.title')}
			>
				{#snippet row(entry: QueueEntry | undefined, index: number)}
					{#if entry}
						{@const track = entry.track}
						<div
							class="item"
							class:current={entry.entryId === current}
							data-index={index}
							role="button"
							tabindex="0"
							title={t('queue.dragHint')}
							ondblclick={() => jump(entry.entryId).catch(() => {})}
							onkeydown={(event) => {
								if (reorderKey(event, index, items.length, reorder)) return;
								if (event.key === 'Enter') jump(entry.entryId).catch(() => {});
								if (event.key === 'Delete') removeQueueEntry(entry.entryId).catch(() => {});
							}}
						>
							<span class="numeric index">{index + 1}</span>
							<span class="text">
								<span class="ellipsis title">{track.title ?? fileName(track.path)}</span>
								<span class="ellipsis muted">{track.artist ?? t('track.unknownArtist')}</span>
							</span>
							<span class="numeric muted time"
								>{clock(track.durationMs === null ? null : track.durationMs / 1000)}</span
							>
							<button
								type="button"
								class="remove"
								aria-label={t('queue.remove')}
								onclick={() => removeQueueEntry(entry.entryId).catch(() => {})}
							>
								<Icon name="close" />
							</button>
						</div>
					{/if}
				{/snippet}
			</VirtualList>
			</div>
		{/if}
	</div>
</aside>

{#if saving}
	<NamePrompt
		title={t('playlist.saveQueue')}
		value={t('playlist.queueName', { date: new Date().toLocaleDateString() })}
		confirm={t('playlist.create')}
		onconfirm={saveQueue}
		oncancel={() => (saving = false)}
	/>
{/if}

<style>
	.queue {
		display: grid;
		grid-template-rows: auto auto 1fr;
		min-height: 0;
		border-left: var(--onsa-hairline) solid var(--onsa-surface-line);
		background: var(--onsa-surface-body);
	}

	.head {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 16px 10px 6px 14px;
	}

	.grow {
		flex: 1;
	}

	h2 {
		margin: 0;
		font-size: 11px;
		font-weight: 500;
	}

	.head span {
		font-size: 11px;
	}

	.tools {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		padding: 0 14px 8px;
	}

	.body {
		min-height: 0;
		display: grid;
	}

	.empty {
		margin: 0;
		padding: 8px 14px;
		font-size: 12.5px;
	}

	.item {
		display: grid;
		grid-template-columns: 26px minmax(0, 1fr) auto 22px;
		align-items: center;
		gap: 8px;
		height: 44px;
		padding: 0 10px 0 12px;
		font-size: 12.5px;
		cursor: default;
		user-select: none;
	}

	.item:hover {
		background: var(--onsa-surface-raised);
	}

	/* Where the row would land, drawn in the gap it is over. */
	.item:global([data-drop='before']) {
		box-shadow: inset 0 2px 0 var(--onsa-role-active);
	}

	.item:global([data-drop='after']) {
		box-shadow: inset 0 -2px 0 var(--onsa-role-active);
	}

	.item:global([data-grabbed]) {
		opacity: 0.5;
	}

	.reorder {
		min-height: 0;
		height: 100%;
	}

	.reorder:global([data-reordering]) {
		cursor: grabbing;
	}

	.item.current .title,
	.item.current .index {
		color: var(--onsa-role-position);
	}

	.index {
		color: var(--onsa-text-secondary);
		text-align: right;
	}

	.text {
		display: grid;
		min-width: 0;
	}

	.time {
		font-size: 11.5px;
	}

	.remove {
		display: grid;
		place-items: center;
		padding: 0;
		border: 0;
		background: none;
		color: var(--onsa-text-secondary);
		opacity: 0;
		cursor: pointer;
	}

	.item:hover .remove,
	.remove:focus-visible {
		opacity: 1;
	}

	.remove:hover {
		color: var(--onsa-role-clip);
	}
</style>
