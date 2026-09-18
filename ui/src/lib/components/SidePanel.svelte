<!--
	The right-hand panel (SPEC section 9.2): the queue, or the words of what
	is playing. Two tabs rather than two panels, because at this width the
	window can only afford one of them at a time.
-->
<script lang="ts">
	import { t } from '$lib/i18n/index.svelte';
	import { closeOverlays, layout, showInPanel } from '$lib/layout.svelte';
	import { player } from '$lib/player.svelte';
	import Icon from './Icon.svelte';
	import LyricsPanel from './LyricsPanel.svelte';
	import QueuePanel from './QueuePanel.svelte';

	interface Props {
		/** Whether the panel is covering the content rather than beside it. */
		floating?: boolean;
	}

	const { floating = false }: Props = $props();

	const tab = $derived(layout.panelTab);
	const items = $derived(player.queue.items);
</script>

<aside class="side" class:floating>
	<header class="head">
		<div class="tabs" role="tablist">
			<button
				type="button"
				class="tab label"
				role="tab"
				aria-selected={tab === 'queue'}
				onclick={() => showInPanel('queue')}>{t('queue.title')}</button
			>
			<button
				type="button"
				class="tab label"
				role="tab"
				aria-selected={tab === 'lyrics'}
				onclick={() => showInPanel('lyrics')}>{t('lyrics.title')}</button
			>
		</div>
		<span class="grow"></span>
		{#if tab === 'queue' && items.length > 0}
			<span class="numeric muted count">{t('queue.count', { n: items.length })}</span>
		{/if}
		{#if floating}
			<button type="button" class="icon-btn" aria-label={t('search.clear')} onclick={closeOverlays}>
				<Icon name="close" />
			</button>
		{/if}
	</header>

	{#if tab === 'queue'}
		<QueuePanel />
	{:else}
		<LyricsPanel />
	{/if}
</aside>

<style>
	.side {
		display: grid;
		grid-template-rows: auto minmax(0, 1fr);
		min-height: 0;
		border-left: var(--onsa-hairline) solid var(--onsa-surface-line);
		background: var(--onsa-surface-body);
	}

	.head {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 10px 6px 14px;
	}

	.tabs {
		display: flex;
		gap: 4px;
	}

	.tab {
		padding: 3px 8px;
		border: 0;
		border-radius: var(--onsa-radius-sm);
		background: none;
		font-size: 11px;
		cursor: pointer;
	}

	.tab:hover {
		color: var(--onsa-role-adjustable);
	}

	/* The tab being shown, marked the way every other "this one" in the
	   window is marked. */
	.tab[aria-selected='true'] {
		background: var(--onsa-surface-well);
		color: var(--onsa-role-active);
	}

	.grow {
		flex: 1;
	}

	.count {
		font-size: 11px;
	}
</style>
