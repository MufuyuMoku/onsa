<!--
  The task bar under a window drawn inside the window (SPEC M16, Jendela
  Biru). Each thing on it does what it looks like it does: the button at
  its start opens the first menu, the song goes to Now Playing, and the
  clock is the real time, changing once a minute.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { navigate } from '$lib/app.svelte';
	import { currentLocale, t } from '$lib/i18n/index.svelte';
	import { askForMenu } from '$lib/menubar.svelte';
	import { player } from '$lib/player.svelte';

	const track = $derived(player.snapshot?.track ?? null);
	const playing = $derived(
		track ? [track.title, track.artist].filter(Boolean).join(' — ') : t('transport.nothing')
	);

	let now = $state(new Date());
	const clock = $derived(
		now.toLocaleTimeString(currentLocale() === 'id' ? 'id-ID' : 'en-GB', {
			hour: '2-digit',
			minute: '2-digit'
		})
	);

	// On the minute, not every so often: the clock never shows a minute
	// that has already gone.
	onMount(() => {
		let timer: ReturnType<typeof setTimeout>;
		const next = () => {
			now = new Date();
			timer = setTimeout(next, 60_000 - (Date.now() % 60_000) + 50);
		};
		next();
		return () => clearTimeout(timer);
	});
</script>

<div class="taskbar">
	<button type="button" class="start" aria-haspopup="menu" onclick={() => askForMenu(0)}>onsa</button>
	<button
		type="button"
		class="song ellipsis"
		title={playing}
		aria-label={`${t('nowPlaying.open')}: ${playing}`}
		onclick={() => navigate({ kind: 'nowPlaying' })}>{playing}</button
	>
	<time class="clock numeric" datetime={now.toISOString()}>{clock}</time>
</div>

<style>
	.taskbar {
		position: absolute;
		left: 0;
		right: 0;
		bottom: 0;
		display: flex;
		align-items: center;
		gap: 8px;
		height: 34px;
		background: var(--onsa-chrome-taskbar);
		box-shadow: inset 0 2px 1px -1px rgb(255 255 255 / 0.4);
		color: var(--onsa-chrome-taskbar-text);
		font-size: 12px;
	}

	.start {
		flex: none;
		height: 100%;
		padding: 0 18px 0 12px;
		border: 0;
		border-radius: 0 12px 12px 0;
		background: var(--onsa-chrome-start);
		box-shadow: inset 0 2px 1px -1px rgb(255 255 255 / 0.45);
		color: var(--onsa-chrome-taskbar-text);
		font: inherit;
		/* Large and bold: what lets the white word sit on the green. */
		font-size: 19px;
		font-weight: 700;
		font-style: italic;
		text-shadow: 1px 1px 1px rgb(0 0 0 / 0.45);
		cursor: pointer;
	}

	.song {
		min-width: 0;
		max-width: 320px;
		height: 24px;
		padding: 0 10px;
		border: 0;
		border-radius: 3px;
		background: color-mix(in srgb, var(--onsa-chrome-frame) 35%, transparent);
		box-shadow: inset 1px 1px 2px rgb(0 0 0 / 0.45);
		color: inherit;
		font: inherit;
		font-weight: 700;
		font-style: italic;
		text-align: left;
		cursor: pointer;
	}

	.clock {
		display: flex;
		align-items: center;
		height: 100%;
		margin-left: auto;
		padding: 0 14px;
		border-left: 1px solid rgb(0 0 0 / 0.35);
		background: var(--onsa-chrome-tray);
		box-shadow: inset 1px 0 0 rgb(255 255 255 / 0.25);
	}
</style>
