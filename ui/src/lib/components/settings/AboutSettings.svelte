<!-- About: version, the log folder, and debug logging for problem reports. -->
<script lang="ts">
	import { failureKey, logOpenFolder, logSetDebug, setCloseToTray } from '$lib/backend';
	import { app } from '$lib/app.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';

	let debug = $state(app.state?.logDebug ?? false);
	let tray = $state(app.state?.closeToTray ?? false);
	let failure = $state<MessageKey | null>(null);

	async function setDebug(enabled: boolean): Promise<void> {
		try {
			await logSetDebug(enabled);
			debug = enabled;
			failure = null;
		} catch (error) {
			failure = failureKey(error);
		}
	}
</script>

<div class="page">
	<section>
		<h2 class="label">{app.info?.name ?? ''}</h2>
		<p class="numeric">{t('about.version', { version: app.info?.version ?? '' })}</p>
	</section>
	<section>
		<h2 class="label">{t('nav.settings')}</h2>
		<label class="check">
			<input
				type="checkbox"
				checked={tray}
				onchange={(event) => {
					const wanted = event.currentTarget.checked;
					setCloseToTray(wanted)
						.then(() => (tray = wanted))
						.catch((error) => (failure = failureKey(error)));
				}}
			/>
			{t('tray.closeToTray')}
		</label>
	</section>
	<section>
		<h2 class="label">{t('about.logs')}</h2>
		<div class="field-row">
			<span>{t('about.logDir')}</span>
			<span class="numeric ellipsis path">{app.state?.logDir ?? ''}</span>
			<button
				type="button"
				class="btn"
				onclick={() => logOpenFolder().catch((error) => (failure = failureKey(error)))}
				>{t('about.openLogs')}</button
			>
		</div>
		<label class="check">
			<input
				type="checkbox"
				checked={debug}
				onchange={(event) => setDebug(event.currentTarget.checked)}
			/>
			{t('about.debug')}
		</label>
		<p class="note muted">{t('about.debugHint')}</p>
		{#if failure}<p class="note fault-text">{t(failure)}</p>{/if}
	</section>
</div>

<style>
	@import './settings.css';

	p {
		margin: 0;
	}

	.path {
		font-size: 12px;
		color: var(--onsa-role-adjustable);
	}
</style>
