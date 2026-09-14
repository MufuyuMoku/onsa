<!--
	Puts everything a context stands for — a whole album, an artist, a genre,
	a folder — into a playlist at once (SPEC section 6.2). The backend works
	out which tracks that is, so nothing has to be listed here first.
-->
<script lang="ts">
	import type { PlayContext } from '$lib/backend';
	import { t } from '$lib/i18n/index.svelte';
	import { addToPlaylist, createPlaylist, playlists } from '$lib/playlists.svelte';
	import Icon from './Icon.svelte';
	import NamePrompt from './NamePrompt.svelte';

	interface Props {
		/** What goes in. */
		context: PlayContext;
		/** Whether there is anything to add at all. */
		enabled?: boolean;
	}

	const { context, enabled = true }: Props = $props();

	let open = $state(false);
	let naming = $state(false);
	let note = $state<string | null>(null);

	// Only manual playlists take tracks: a smart one is its rules.
	const manual = $derived(playlists.all.filter((playlist) => playlist.kind === 'manual'));

	async function into(id: number, name: string): Promise<void> {
		open = false;
		try {
			const added = await addToPlaylist(id, context);
			note = t('playlist.addedTo', { n: added, name });
		} catch {
			note = null;
		}
	}

	async function intoNew(name: string): Promise<void> {
		naming = false;
		try {
			await into(await createPlaylist(name), name);
		} catch {
			note = null;
		}
	}
</script>

<div class="wrap">
	<button
		type="button"
		class="btn"
		disabled={!enabled}
		aria-expanded={open}
		onclick={() => (open = !open)}
	>
		<Icon name="playlist" />
		{t('playlist.addTo')}
	</button>
	{#if open}
		<!-- Clicking anywhere else puts the menu away. -->
		<div class="backdrop" role="presentation" onclick={() => (open = false)}></div>
		<menu class="menu">
			{#each manual as playlist (playlist.id)}
				<li>
					<button type="button" class="ellipsis" onclick={() => into(playlist.id, playlist.name)}
						>{playlist.name}</button
					>
				</li>
			{/each}
			<li class="new">
				<button
					type="button"
					onclick={() => {
						open = false;
						naming = true;
					}}><Icon name="plus" />{t('playlist.new')}</button
				>
			</li>
		</menu>
	{/if}
</div>

{#if note}
	<p class="muted note">{note}</p>
{/if}

{#if naming}
	<NamePrompt
		title={t('playlist.new')}
		value={t('playlist.untitled')}
		confirm={t('playlist.create')}
		onconfirm={intoNew}
		oncancel={() => (naming = false)}
	/>
{/if}

<style>
	.wrap {
		position: relative;
		display: inline-block;
	}

	.btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}

	.btn :global(svg) {
		width: 14px;
		height: 14px;
	}

	.backdrop {
		position: fixed;
		inset: 0;
		z-index: 10;
	}

	.menu {
		position: absolute;
		z-index: 11;
		top: calc(100% + 4px);
		left: 0;
		display: grid;
		max-height: min(50vh, 320px);
		max-width: 260px;
		margin: 0;
		padding: 4px;
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-raised);
		box-shadow: 0 10px 24px rgb(0 0 0 / 0.45);
		overflow-y: auto;
		scrollbar-width: thin;
		list-style: none;
	}

	.menu button {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 6px 14px 6px 10px;
		border: 0;
		border-radius: var(--onsa-radius-sm);
		background: none;
		color: var(--onsa-text-primary);
		font: inherit;
		font-size: 13px;
		white-space: nowrap;
		text-align: left;
		cursor: pointer;
	}

	.menu button:hover {
		background: var(--onsa-surface-body);
	}

	.menu :global(svg) {
		width: 13px;
		height: 13px;
		flex: none;
	}

	.new {
		margin-top: 4px;
		border-top: var(--onsa-hairline) solid var(--onsa-surface-line);
	}

	.note {
		margin: 6px 0 0;
		font-size: 12px;
	}
</style>
