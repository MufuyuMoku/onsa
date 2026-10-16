<!--
  A menu bar, for a layout that draws a window inside the window (SPEC M16,
  Jendela Biru). Every entry runs something Onsa already does; nothing is
  here for the look alone. It follows the ARIA menu bar pattern: the arrows
  move along the bar and down a menu, Enter or Space opens and chooses,
  Escape closes, and Alt with the underlined letter opens a menu from
  anywhere in the window. Alt on its own does nothing, so a window manager
  that wants it keeps it.
-->
<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { cycleRepeat, toggleShuffle, player } from '$lib/player.svelte';
	import { nextTrack, previousTrack, togglePlay } from '$lib/backend';
	import { app, navigate, setMiniPlayer } from '$lib/app.svelte';
	import { chooseLayout, layoutChoice, PRESETS, type LayoutChoice } from '$lib/arrangement.svelte';
	import { toggleHelp, toggleQueue } from '$lib/layout.svelte';
	import { showPlaying } from '$lib/places.svelte';
	import { menuAsked } from '$lib/menubar.svelte';
	import { settings } from '$lib/settings.svelte';
	import { activeTheme } from '$lib/theme/index.svelte';
	import { currentLocale, t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';

	type Entry = {
		/** What it says. */
		label: string;
		/** The keys that do the same, where there are any (SPEC §13). */
		keys?: string;
		/** Whether it is the one in force, for a choice among several. */
		checked?: boolean;
		run: () => void;
	};
	type Menu = { name: MessageKey; entries: () => (Entry | 'rule')[] };

	const presetKeys: Record<(typeof PRESETS)[number], MessageKey> = {
		classic: 'layout.classic',
		gallery: 'layout.gallery',
		fullscreen: 'layout.fullscreen'
	};

	const quietly = (work: () => Promise<unknown>) => () => void work().catch(() => {});

	const menus: Menu[] = [
		{
			name: 'menu.file',
			entries: () => [
				{ label: t('nav.tracks'), run: () => navigate({ kind: 'tracks' }) },
				{ label: t('nav.albums'), run: () => navigate({ kind: 'albums' }) },
				{ label: t('nav.playlists'), run: () => navigate({ kind: 'playlists' }) },
				'rule',
				{ label: t('nav.downloads'), run: () => navigate({ kind: 'downloads' }) }
			]
		},
		{
			name: 'menu.play',
			entries: () => [
				{
					label: t(player.snapshot?.state === 'playing' ? 'transport.pause' : 'transport.play'),
					keys: t('menu.keySpace'),
					run: quietly(togglePlay)
				},
				{ label: t('transport.previous'), keys: 'Ctrl+←', run: quietly(previousTrack) },
				{ label: t('transport.next'), keys: 'Ctrl+→', run: quietly(nextTrack) },
				'rule',
				{ label: t('queue.shuffle'), checked: player.snapshot?.shuffle ?? false, run: quietly(toggleShuffle) },
				{ label: t(repeatKey()), run: quietly(cycleRepeat) },
				'rule',
				{ label: t('menu.toPlaying'), keys: 'Ctrl+J', run: showPlaying }
			]
		},
		{
			name: 'menu.view',
			entries: () => [
				{ label: t('nowPlaying.open'), keys: 'Ctrl+N', run: () => navigate({ kind: 'nowPlaying' }) },
				{ label: t('queue.title'), run: toggleQueue },
				'rule',
				...layoutEntries(),
				'rule',
				{ label: t('mini.open'), keys: 'Ctrl+M', run: () => void setMiniPlayer(true) }
			]
		},
		{
			name: 'menu.tools',
			entries: () => [
				{ label: t('nav.tidy'), run: () => navigate({ kind: 'tidy' }) },
				{ label: t('settings.dsp'), run: () => navigate({ kind: 'settings', section: 'dsp' }) },
				{
					label: t('nav.settings'),
					keys: 'Ctrl+,',
					run: () => navigate({ kind: 'settings', section: 'output' })
				}
			]
		},
		{
			name: 'menu.help',
			entries: () => [
				{ label: t('help.open'), run: toggleHelp },
				{ label: t('nav.help'), run: () => navigate({ kind: 'help' }) },
				{ label: t('settings.about'), run: () => navigate({ kind: 'settings', section: 'about' }) }
			]
		}
	];

	function repeatKey(): MessageKey {
		const mode = settings.value?.playback.repeat;
		return mode === 'all' ? 'queue.repeatAll' : mode === 'one' ? 'queue.repeatOne' : 'queue.repeatOff';
	}

	function layoutEntries(): Entry[] {
		const own = activeTheme()?.signature;
		const choices: [LayoutChoice, string][] = [
			...(own ? [['signature', own.name[currentLocale()]] as [LayoutChoice, string]] : []),
			...PRESETS.map((preset) => [preset, t(presetKeys[preset])] as [LayoutChoice, string])
		];
		return choices.map(([choice, label]) => ({
			label,
			checked: layoutChoice() === choice,
			run: () => chooseLayout(choice)
		}));
	}

	/**
	 * A name with its access key marked by `&`, as in "&Berkas": the letter
	 * after it is underlined and opens the menu with Alt.
	 */
	function split(name: string): { before: string; key: string; after: string } {
		const at = name.indexOf('&');
		if (at < 0 || at === name.length - 1) return { before: name, key: '', after: '' };
		return { before: name.slice(0, at), key: name[at + 1], after: name.slice(at + 2) };
	}

	let open = $state<number | null>(null);
	/** Which top entry takes the focus when the bar is tabbed into. */
	let current = $state(0);
	let bar: HTMLElement | undefined = $state();
	const tops: HTMLButtonElement[] = $state([]);

	async function openMenu(index: number, focus: 'first' | 'last' | 'none' = 'first'): Promise<void> {
		open = index;
		current = index;
		await tick();
		const items = itemsOf(index);
		if (focus === 'first') items[0]?.focus();
		if (focus === 'last') items.at(-1)?.focus();
	}

	function closeMenu(refocus: boolean): void {
		const was = open;
		open = null;
		if (refocus && was !== null) tops[was]?.focus();
	}

	function itemsOf(index: number): HTMLElement[] {
		return [...(bar?.querySelectorAll<HTMLElement>(`#menu-${index} [role^='menuitem']`) ?? [])];
	}

	function choose(entry: Entry): void {
		closeMenu(false);
		entry.run();
	}

	function onTopKey(event: KeyboardEvent, index: number): void {
		const count = menus.length;
		if (event.key === 'ArrowRight' || event.key === 'ArrowLeft') {
			event.preventDefault();
			const next = (index + (event.key === 'ArrowRight' ? 1 : count - 1)) % count;
			current = next;
			if (open !== null) void openMenu(next, 'none');
			tops[next]?.focus();
		} else if (event.key === 'ArrowDown' || event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			event.stopPropagation();
			void openMenu(index, 'first');
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			void openMenu(index, 'last');
		} else if (event.key === 'Escape' && open !== null) {
			event.preventDefault();
			closeMenu(true);
		}
	}

	function onItemKey(event: KeyboardEvent, index: number): void {
		const items = itemsOf(index);
		const at = items.indexOf(event.target as HTMLElement);
		if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
			event.preventDefault();
			const step = event.key === 'ArrowDown' ? 1 : items.length - 1;
			items[(at + step) % items.length]?.focus();
		} else if (event.key === 'Home' || event.key === 'End') {
			event.preventDefault();
			(event.key === 'Home' ? items[0] : items.at(-1))?.focus();
		} else if (event.key === 'Escape') {
			event.preventDefault();
			closeMenu(true);
		} else if (event.key === 'ArrowRight' || event.key === 'ArrowLeft') {
			event.preventDefault();
			const count = menus.length;
			void openMenu((index + (event.key === 'ArrowRight' ? 1 : count - 1)) % count, 'first');
		} else if (event.key === 'Tab') {
			closeMenu(false);
		} else if (event.key === ' ') {
			// Space chooses here, rather than playing or pausing.
			event.stopPropagation();
		}
	}

	// The task bar's button asks for the first menu. Only a request made
	// while this bar is here counts: one left from before the layout was
	// last put on is not a reason to open a menu nobody asked for now.
	let answered = menuAsked()?.at ?? null;
	$effect(() => {
		const asked = menuAsked();
		if (!asked || asked.at === answered) return;
		answered = asked.at;
		void openMenu(asked.index, 'first');
	});

	// Alt with a menu's letter opens it, from anywhere in the window; Alt
	// alone is left to the system. A menu that is open closes when the
	// pointer goes down anywhere else.
	onMount(() => {
		const onKey = (event: KeyboardEvent) => {
			if (!event.altKey || event.ctrlKey || event.metaKey || event.key.length !== 1) return;
			if (app.mini) return;
			const letter = event.key.toLowerCase();
			const index = menus.findIndex((menu) => split(t(menu.name)).key.toLowerCase() === letter);
			if (index < 0) return;
			event.preventDefault();
			void openMenu(index, 'first');
		};
		const onDown = (event: PointerEvent) => {
			if (open !== null && !bar?.contains(event.target as Node)) closeMenu(false);
		};
		window.addEventListener('keydown', onKey);
		window.addEventListener('pointerdown', onDown);
		return () => {
			window.removeEventListener('keydown', onKey);
			window.removeEventListener('pointerdown', onDown);
		};
	});
</script>

<div class="menubar" role="menubar" aria-label={t('menu.bar')} bind:this={bar}>
	{#each menus as menu, index (menu.name)}
		{@const name = split(t(menu.name))}
		<div class="top">
			<button
				type="button"
				role="menuitem"
				class="name"
				class:open={open === index}
				aria-haspopup="menu"
				aria-expanded={open === index}
				aria-controls="menu-{index}"
				aria-keyshortcuts={name.key ? `Alt+${name.key.toUpperCase()}` : undefined}
				tabindex={current === index ? 0 : -1}
				bind:this={tops[index]}
				onclick={() => (open === index ? closeMenu(false) : void openMenu(index, 'none'))}
				onpointerenter={() => open !== null && open !== index && void openMenu(index, 'none')}
				onkeydown={(event) => onTopKey(event, index)}
				>{name.before}<u>{name.key}</u>{name.after}</button
			>
			{#if open === index}
				<div class="menu" id="menu-{index}" role="menu" aria-label={name.before + name.key + name.after}>
					{#each menu.entries() as entry, at (at)}
						{#if entry === 'rule'}
							<div class="rule" role="separator"></div>
						{:else}
							<button
								type="button"
								role={entry.checked === undefined ? 'menuitem' : 'menuitemcheckbox'}
								aria-checked={entry.checked}
								class="item"
								tabindex="-1"
								onclick={() => choose(entry)}
								onkeydown={(event) => onItemKey(event, index)}
							>
								<span class="tick" aria-hidden="true">{entry.checked ? '✔' : ''}</span>
								<span class="words">{entry.label}</span>
								{#if entry.keys}<span class="keys">{entry.keys}</span>{/if}
							</button>
						{/if}
					{/each}
				</div>
			{/if}
		</div>
	{/each}
</div>

<style>
	.menubar {
		position: relative;
		z-index: 40;
		display: flex;
		align-items: stretch;
		gap: 2px;
		height: 22px;
		padding: 0 4px;
		border-bottom: 1px solid var(--onsa-edge-light);
		background: var(--onsa-surface-body);
		color: var(--onsa-text-primary);
		font-size: 12px;
	}

	.top {
		position: relative;
		display: flex;
	}

	.name {
		padding: 0 7px;
		border: 0;
		background: none;
		color: inherit;
		font: inherit;
		cursor: default;
	}

	.name:hover,
	.name.open,
	.name:focus-visible {
		background: var(--onsa-lit-highlight);
		color: var(--onsa-lit-highlight-text);
		outline: none;
	}

	.menu {
		position: absolute;
		top: 100%;
		left: 0;
		display: grid;
		min-width: 220px;
		padding: 2px;
		border: 1px solid var(--onsa-edge-dark);
		background: var(--onsa-surface-well);
		box-shadow: 2px 2px 3px rgb(0 0 0 / 0.35);
	}

	.item {
		display: grid;
		grid-template-columns: 16px minmax(0, 1fr) auto;
		align-items: center;
		gap: 6px;
		min-height: 22px;
		padding: 0 10px 0 4px;
		border: 0;
		background: none;
		color: var(--onsa-text-primary);
		font: inherit;
		text-align: left;
		cursor: default;
	}

	.item:hover,
	.item:focus-visible {
		background: var(--onsa-lit-highlight);
		color: var(--onsa-lit-highlight-text);
		outline: none;
	}

	.keys {
		padding-left: 18px;
	}

	.rule {
		height: 1px;
		margin: 3px 2px;
		background: var(--onsa-surface-line);
	}

	u {
		text-decoration-thickness: 1px;
	}
</style>
