<!--
	The help page: the way into every page's own help, and the things that
	belong to no single page — where the words to a song come from, what a
	theme file holds, when Onsa touches a file, which outside programs it
	uses, what this version cannot do, and how to report a fault.
-->
<script lang="ts">
	import { app, navigate, showFirstRun, type SettingsSection, type View } from '$lib/app.svelte';
	import { openHelp } from '$lib/layout.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { TOPICS } from '$lib/help/topics';

	/**
	 * The example theme. It is a file's contents rather than wording, the
	 * same in both languages, so it lives here and not in the dictionary.
	 */
	const THEME_EXAMPLE = `{
  "id": "tema-saya",
  "name": { "id": "Tema saya", "en": "My theme" },
  "dark": true,
  "color": {
    "surface": { "app": "#0d1117", "body": "#161b22", "raised": "#1c232c" },
    "text": { "primary": "#e6edf3", "secondary": "#8b949e" },
    "role": {
      "label": "#8b949e",
      "adjustable": "#79c0ff",
      "active": "#3fb950",
      "position": "#d29922",
      "caution": "#db6d28",
      "clip": "#f85149"
    }
  }
}`;

	let copied = $state(false);

	/** Where a topic sits in the window, for the contents to point at. */
	function whereIs(id: string): View | null {
		if (id.startsWith('settings.')) {
			return { kind: 'settings', section: id.slice('settings.'.length) as SettingsSection };
		}
		// Three topics are the window's own furniture and the search
		// results, which have no page of their own to open.
		if (['header', 'transport', 'panel', 'search'].includes(id)) return null;
		if (id === 'album' || id === 'artist' || id === 'genre' || id === 'folder') return null;
		if (id === 'playlist') return { kind: 'playlists' };
		return { kind: id } as View;
	}

	function go(id: string): void {
		const where = whereIs(id);
		if (where) navigate(where);
		// Whatever was asked for is shown, even when it is the window's own
		// furniture and there is nowhere to go.
		openHelp(id);
	}

	async function copyExample(): Promise<void> {
		try {
			await navigator.clipboard.writeText(THEME_EXAMPLE);
			copied = true;
		} catch {
			// A clipboard the system will not open is not worth a fault
			// message: the text is on the screen to select by hand.
		}
	}
</script>

<div class="help-page">
	<h1>{t('nav.help')}</h1>
	<p class="intro">{t('help.page.intro')}</p>

	<section>
		<h2 class="label">{t('help.page.contents')}</h2>
		<ul class="contents">
			{#each TOPICS as one (one.id)}
				{#if one.id !== 'help'}
					<li>
						<button type="button" class="go" onclick={() => go(one.id)}>{t(one.title)}</button>
						<span class="muted">{t(one.what)}</span>
					</li>
				{/if}
			{/each}
		</ul>
	</section>

	<section>
		<h2 class="label">{t('help.page.things')}</h2>

		<article>
			<h3>{t('help.lyrics.head')}</h3>
			<p>{t('help.lyrics.body')}</p>
			<p>{t('help.lyrics.offset')}</p>
		</article>

		<article>
			<h3>{t('help.theme.head')}</h3>
			<p>{t('help.theme.body')}</p>
			<h4 class="label">{t('help.theme.example')}</h4>
			<pre class="numeric">{THEME_EXAMPLE}</pre>
			<div class="row">
				<button type="button" class="btn" onclick={copyExample}>{t('help.theme.copy')}</button>
				{#if copied}<span class="muted">{t('help.theme.copied')}</span>{/if}
			</div>
		</article>

		<article>
			<h3>{t('help.write.head')}</h3>
			<p>{t('help.write.body')}</p>
			<p>{t('help.write.never')}</p>
		</article>

		<article>
			<h3>{t('help.programs.head')}</h3>
			<p>{t('help.programs.body')}</p>
		</article>

		<article>
			<h3>{t('help.limits.head')}</h3>
			<ul>
				<li>{t('help.limits.opus')}</li>
				<li>{t('help.limits.folder')}</li>
				<li>{t('help.limits.lastfm')}</li>
			</ul>
		</article>

		<article>
			<h3>{t('help.log.head')}</h3>
			<p>{t('help.log.body')}</p>
		</article>

		<article>
			<h3>{t('help.firstRun.head')}</h3>
			<p>{t('help.firstRun.body')}</p>
			<button type="button" class="btn" onclick={showFirstRun}>{t('help.firstRun')}</button>
		</article>
	</section>

	{#if app.view.kind === 'help'}
		<p class="muted small">{t('help.elsewhere')}</p>
	{/if}
</div>

<style>
	.help-page {
		display: grid;
		align-content: start;
		gap: 18px;
		height: 100%;
		padding: 4px 18px 24px;
		overflow-y: auto;
		scrollbar-width: thin;
	}

	h1 {
		margin: 0;
		font-size: 20px;
		font-weight: 500;
	}

	h2 {
		margin: 0 0 8px;
		font-size: 11px;
		font-weight: 500;
	}

	h3 {
		margin: 0 0 6px;
		font-size: 14px;
		font-weight: 500;
		color: var(--onsa-role-active);
	}

	h4 {
		margin: 10px 0 4px;
		font-size: 11px;
		font-weight: 500;
	}

	p {
		margin: 0 0 8px;
		max-width: 76ch;
		font-size: 13px;
		line-height: 1.6;
		color: var(--onsa-text-secondary);
	}

	.intro {
		color: var(--onsa-text-primary);
	}

	.small {
		font-size: 11.5px;
	}

	section {
		display: grid;
		gap: 16px;
	}

	article {
		display: grid;
		justify-items: start;
	}

	.contents {
		display: grid;
		gap: 8px;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.contents li {
		display: grid;
		grid-template-columns: minmax(0, 14em) minmax(0, 1fr);
		gap: 12px;
		align-items: baseline;
		max-width: 92ch;
	}

	.contents .muted {
		font-size: 12.5px;
		line-height: 1.5;
		overflow: hidden;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
	}

	.go {
		padding: 0;
		border: 0;
		background: none;
		color: var(--onsa-role-adjustable);
		font: inherit;
		text-align: left;
		text-decoration: underline;
		text-underline-offset: 3px;
		cursor: pointer;
	}

	.go:hover {
		color: var(--onsa-role-active);
	}

	ul:not(.contents) {
		margin: 0;
		padding-left: 18px;
		max-width: 76ch;
		font-size: 13px;
		line-height: 1.6;
		color: var(--onsa-text-secondary);
	}

	ul:not(.contents) li + li {
		margin-top: 8px;
	}

	pre {
		margin: 0 0 8px;
		padding: 12px 14px;
		max-width: 76ch;
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		border-radius: var(--onsa-radius-md);
		background: var(--onsa-surface-raised);
		color: var(--onsa-text-secondary);
		font-size: 12px;
		line-height: 1.5;
		overflow-x: auto;
	}

	.row {
		display: flex;
		align-items: center;
		gap: 10px;
	}
</style>
