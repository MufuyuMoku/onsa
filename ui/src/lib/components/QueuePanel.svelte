<!-- The queue (SPEC section 6.1): what plays next; double-click to jump. -->
<script lang="ts">
	import { jump, type Track } from '$lib/backend';
	import { clock } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import { player } from '$lib/player.svelte';
	import VirtualList from './VirtualList.svelte';

	const ROW = 44;

	const items = $derived(player.queue.items);
	const current = $derived(player.snapshot?.current ?? null);
	const load = $derived.by(() => {
		const list = items;
		return (offset: number, limit: number) => Promise.resolve(list.slice(offset, offset + limit));
	});

	let list: VirtualList<Track> | undefined = $state();
	$effect(() => {
		if (current !== null) list?.reveal(current);
	});

	function fileName(path: string): string {
		return path.split(/[\\/]/).pop() ?? path;
	}
</script>

<aside class="queue">
	<header class="head">
		<h2 class="label">{t('queue.title')}</h2>
		{#if items.length > 0}<span class="numeric muted">{t('queue.count', { n: items.length })}</span>{/if}
	</header>
	<div class="body">
		{#if items.length === 0}
			<p class="muted empty">{t('queue.empty')}</p>
		{:else}
			<VirtualList
				bind:this={list}
				count={items.length}
				rowHeight={ROW}
				{load}
				version={items}
				label={t('queue.title')}
			>
				{#snippet row(track: Track | undefined, index: number)}
					{#if track}
						<div
							class="item"
							class:current={index === current}
							role="button"
							tabindex="0"
							title={t('track.playHint')}
							ondblclick={() => jump(index).catch(() => {})}
							onkeydown={(event) => {
								if (event.key === 'Enter') jump(index).catch(() => {});
							}}
						>
							<span class="numeric index">{index + 1}</span>
							<span class="text">
								<span class="ellipsis title">{track.title ?? fileName(track.path)}</span>
								<span class="ellipsis muted">{track.artist ?? t('track.unknownArtist')}</span>
							</span>
							<span class="numeric muted"
								>{clock(track.durationMs === null ? null : track.durationMs / 1000)}</span
							>
						</div>
					{/if}
				{/snippet}
			</VirtualList>
		{/if}
	</div>
</aside>

<style>
	.queue {
		display: grid;
		grid-template-rows: auto 1fr;
		min-height: 0;
		border-left: var(--onsa-hairline) solid var(--onsa-surface-line);
		background: var(--onsa-surface-body);
	}

	.head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		padding: 16px 14px 8px;
	}

	h2 {
		margin: 0;
		font-size: 11px;
		font-weight: 500;
	}

	.head span {
		font-size: 11px;
	}

	.body {
		min-height: 0;
	}

	.empty {
		margin: 0;
		padding: 8px 14px;
		font-size: 12.5px;
	}

	.item {
		display: grid;
		grid-template-columns: 28px minmax(0, 1fr) auto;
		align-items: center;
		gap: 8px;
		height: 44px;
		padding: 0 12px;
		font-size: 12.5px;
		cursor: default;
		user-select: none;
	}

	.item:hover {
		background: var(--onsa-surface-raised);
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
</style>
