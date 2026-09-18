<!--
	The words of whatever is playing (SPEC section 10): the line being sung
	lit, a click on a line to jump there, and the words scrolling themselves
	while the listener is not scrolling them.
-->
<script lang="ts">
	import { seek } from '$lib/backend';
	import { t } from '$lib/i18n/index.svelte';
	import {
		lineAt,
		lookAgain,
		lyrics,
		nudge,
		NUDGE_MS,
		resetOffset,
		saveBeside
	} from '$lib/lyrics.svelte';
	import { player } from '$lib/player.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';

	interface Props {
		/** Whether this is the big one on the Now Playing screen. */
		large?: boolean;
	}

	const { large = false }: Props = $props();

	/** What each source is called, in the listener's own language. */
	const SOURCE: Record<string, MessageKey> = {
		edited: 'lyrics.source.edited',
		file: 'lyrics.source.file',
		tag: 'lyrics.source.tag',
		lrclib: 'lyrics.source.lrclib'
	};

	/** How long the listener's own scrolling holds the words still. */
	const THEIRS_MS = 6000;

	const words = $derived(lyrics.words);
	const playing = $derived(player.snapshot?.track != null);
	const active = $derived(lineAt(words, player.position));

	const shift = $derived.by(() => {
		const ms = words?.offsetMs ?? 0;
		if (ms === 0) return null;
		const seconds = (ms / 1000).toFixed(2);
		return t('lyrics.offsetValue', { seconds: ms > 0 ? `+${seconds}` : seconds });
	});

	/** What to say when there is nothing to show. */
	const nothing = $derived.by(() => {
		if (!playing) return 'lyrics.nothingPlaying';
		if (words?.looking) return 'lyrics.looking';
		return 'lyrics.none';
	});

	/**
	 * Whether to add that the service is off.
	 *
	 * It is the second thing a listener wants to know, after whether there
	 * are any words at all — and only when a service could have been asked.
	 */
	const alsoOffline = $derived(
		playing && !words?.looking && words?.canAsk === true && words?.online === false
	);

	let box: HTMLDivElement | undefined = $state();
	let theirsUntil = 0;

	/** The listener scrolling wins over the words scrolling themselves. */
	function theirs(): void {
		theirsUntil = Date.now() + THEIRS_MS;
	}

	$effect(() => {
		const index = active;
		if (index < 0 || !box || Date.now() < theirsUntil) return;
		const line = box.querySelector<HTMLElement>(`[data-line="${index}"]`);
		line?.scrollIntoView({ block: 'center', behavior: 'smooth' });
	});

	function jumpTo(atMs: number | null): void {
		if (atMs === null || !words) return;
		seek(Math.max(0, (atMs + words.offsetMs) / 1000)).catch(() => {});
	}
</script>

<div class="lyrics" class:large>
	<div class="tools">
		{#if words && words.lines.length > 0 && SOURCE[words.source]}
			<span class="chip flat">{t(SOURCE[words.source])}</span>
			{#if !words.synced}
				<span class="chip flat muted">{t('lyrics.unsynced')}</span>
			{/if}
		{/if}
		{#if words?.synced}
			<button
				type="button"
				class="chip"
				aria-label={t('lyrics.earlier')}
				title={t('lyrics.earlier')}
				onclick={() => nudge(-NUDGE_MS)}>−</button
			>
			<button
				type="button"
				class="chip"
				aria-label={t('lyrics.later')}
				title={t('lyrics.later')}
				onclick={() => nudge(NUDGE_MS)}>+</button
			>
			{#if shift}
				<button
					type="button"
					class="chip"
					title={t('lyrics.resetOffset')}
					onclick={() => resetOffset()}>{shift}</button
				>
			{/if}
		{/if}
		<span class="grow"></span>
		{#if words?.online && words.canAsk}
			<button type="button" class="chip" disabled={lyrics.busy} onclick={() => lookAgain()}>
				{t('lyrics.lookAgain')}
			</button>
		{/if}
		{#if words && words.lines.length > 0 && !words.beside}
			<button type="button" class="chip" disabled={lyrics.busy} onclick={() => saveBeside()}>
				{t('lyrics.saveBeside')}
			</button>
		{/if}
	</div>

	<!-- The role is here because the words answer the wheel: the listener
	     scrolling holds them still for a moment rather than being fought. -->
	<div class="body" role="group" bind:this={box} onwheel={theirs} ontouchmove={theirs}>
		{#if !words || words.lines.length === 0}
			<p class="muted empty">{t(nothing)}</p>
			{#if alsoOffline}
				<p class="muted empty">{t('lyrics.offline')}</p>
			{/if}
		{:else if words.synced}
			<ol class="lines" title={t('lyrics.seekHint')}>
				{#each words.lines as line, index (index)}
					<li data-line={index}>
						<button
							type="button"
							class="line"
							class:on={index === active}
							class:untimed={line.atMs === null}
							disabled={line.atMs === null}
							onclick={() => jumpTo(line.atMs)}
						>
							{line.text || '♪'}
						</button>
					</li>
				{/each}
			</ol>
		{:else}
			<div class="plain">
				{#each words.lines as line, index (index)}
					<p class:blank={!line.text.trim()}>{line.text}</p>
				{/each}
			</div>
		{/if}
	</div>
</div>

<style>
	.lyrics {
		display: grid;
		grid-template-rows: auto minmax(0, 1fr);
		min-height: 0;
	}

	.tools {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px;
		padding: 0 14px 8px;
	}

	.grow {
		flex: 1;
	}

	/* A chip that says something rather than doing something. */
	.chip.flat {
		border-color: transparent;
		background: var(--onsa-surface-well);
		cursor: default;
	}

	.body {
		min-height: 0;
		overflow-y: auto;
		padding: 0 14px 18px;
		scrollbar-gutter: stable;
	}

	.empty {
		margin: 0;
		padding: 8px 0;
		font-size: 12.5px;
	}

	.lines {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.line {
		display: block;
		width: 100%;
		padding: 4px 6px;
		border: 0;
		border-radius: var(--onsa-radius-sm);
		background: none;
		/* A block of words is text, not an indicator: the lit colours are
		   for the things that are lit. */
		color: var(--onsa-text-secondary);
		font: inherit;
		font-size: 13px;
		line-height: 1.5;
		text-align: left;
		cursor: pointer;
		transition: color var(--onsa-motion-fast) var(--onsa-motion-ease);
	}

	.line:hover:not(:disabled) {
		background: var(--onsa-surface-well);
		color: var(--onsa-role-adjustable);
	}

	.line.untimed {
		cursor: default;
		opacity: 0.75;
	}

	/* The line being sung. Its colour is the one that means "here", the
	   same as the position everywhere else in the window. */
	.line.on {
		color: var(--onsa-role-position);
		font-weight: 500;
	}

	:global(:root[data-onsa-glow='true']) .line.on {
		text-shadow: 0 0 10px var(--onsa-lit-glow);
	}

	.plain p {
		margin: 0;
		padding: 2px 6px;
		font-size: 13px;
		line-height: 1.55;
		color: var(--onsa-text-secondary);
	}

	.plain p.blank {
		height: 0.6em;
	}

	/* On the Now Playing screen the words are the point, not a sidebar:
	   they line up with everything else on the page rather than sitting in
	   a panel's own margin. */
	.large .tools,
	.large .body {
		padding-left: 0;
		padding-right: 0;
	}

	.large .line,
	.large .plain p {
		font-size: 16px;
		line-height: 1.7;
	}

	.large .line.on {
		color: var(--onsa-role-position);
	}
</style>
