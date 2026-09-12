<!--
	The signal path strip (SPEC section 9.2): the chain the audio really goes
	through. Click a stage to switch it on or off; right-click to open its
	settings.
-->
<script lang="ts">
	import type { RgMode } from '$lib/backend';
	import { navigate, type SettingsSection } from '$lib/app.svelte';
	import { db, rate } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import { player } from '$lib/player.svelte';
	import { settings, updateDsp } from '$lib/settings.svelte';

	const signal = $derived(player.snapshot?.signal ?? null);
	const dsp = $derived(settings.value?.dsp ?? null);

	/** The mode ReplayGain returns to when switched back on. */
	let lastMode = $state<Exclude<RgMode, 'off'>>('auto');
	$effect(() => {
		if (dsp && dsp.replaygainMode !== 'off') lastMode = dsp.replaygainMode;
	});

	function open(section: SettingsSection, event: MouseEvent): void {
		event.preventDefault();
		navigate({ kind: 'settings', section });
	}

	const source = $derived.by(() => {
		const stage = signal?.source;
		if (!stage) return null;
		const parts = [stage.codec ?? '', rate(stage.sampleRate)];
		if (stage.bitDepth) parts.push(`${stage.bitDepth}-bit`);
		return parts.filter(Boolean).join(' ');
	});

	const rgDetail = $derived.by(() => {
		const stage = signal?.replaygain;
		if (!stage || stage.mode === 'off') return '';
		const which = stage.album ? t('signal.album') : t('signal.track');
		return stage.gainDb === null ? which : `${which} ${db(stage.gainDb)}`;
	});

	const eqDetail = $derived.by(() => {
		const stage = signal?.eq;
		if (!stage) return '';
		const bands =
			stage.kind === 'graphic'
				? t('signal.bands', { n: stage.bands })
				: t('signal.filters', { n: stage.bands });
		return Math.abs(stage.preampDb) >= 0.05 ? `${bands} · ${db(stage.preampDb)}` : bands;
	});
</script>

<div class="path" role="group" aria-label={t('signal.label')} title={t('signal.hint')}>
	<span class="stage static" class:on={source !== null}>{source ?? t('signal.idle')}</span>

	{#if signal?.resample}
		<span class="sep"></span>
		<button
			type="button"
			class="stage on"
			onclick={(event) => open('output', event)}
			oncontextmenu={(event) => open('output', event)}
		>
			{t('signal.resample')}<em>{rate(signal.resample.to)}</em>
		</button>
	{/if}

	<span class="sep"></span>
	<button
		type="button"
		class="stage"
		class:on={dsp?.replaygainMode !== 'off'}
		aria-pressed={dsp?.replaygainMode !== 'off'}
		disabled={!dsp}
		onclick={() => updateDsp({ replaygainMode: dsp?.replaygainMode === 'off' ? lastMode : 'off' })}
		oncontextmenu={(event) => open('output', event)}
	>
		{t('signal.replaygain')}{#if rgDetail}<em>{rgDetail}</em>{/if}
	</button>

	<span class="sep"></span>
	<button
		type="button"
		class="stage"
		class:on={dsp?.eqEnabled}
		aria-pressed={dsp?.eqEnabled ?? false}
		disabled={!dsp}
		onclick={() => updateDsp({ eqEnabled: !dsp?.eqEnabled })}
		oncontextmenu={(event) => open('dsp', event)}
	>
		{t('signal.eq')}<em>{eqDetail}</em>
	</button>

	<span class="sep"></span>
	<button
		type="button"
		class="stage"
		class:on={dsp?.limiterEnabled}
		aria-pressed={dsp?.limiterEnabled ?? false}
		disabled={!dsp}
		onclick={() => updateDsp({ limiterEnabled: !dsp?.limiterEnabled })}
		oncontextmenu={(event) => open('dsp', event)}
	>
		{t('signal.limiter')}
	</button>

	<span class="sep"></span>
	<button
		type="button"
		class="stage"
		class:on={signal?.output != null}
		onclick={(event) => open('output', event)}
		oncontextmenu={(event) => open('output', event)}
	>
		{#if signal?.output}
			{signal.output.backend}<em>{signal.output.name} · {rate(signal.output.sampleRate)}</em>
		{:else}
			{t('output.none')}
		{/if}
	</button>
</div>

<style>
	.path {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px 8px;
		padding: 8px 16px 10px;
		font-family: var(--onsa-font-label);
		font-size: 12px;
		color: var(--onsa-role-label);
		background: var(--onsa-surface-body);
	}

	:global(:root[data-onsa-label-case='uppercase']) .path {
		text-transform: uppercase;
		letter-spacing: 0.06em;
	}

	.sep {
		width: 14px;
		height: var(--onsa-hairline);
		background: var(--onsa-role-label);
		opacity: 0.5;
	}

	.stage {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 3px 9px;
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		border-radius: 3px;
		background: none;
		color: var(--onsa-text-secondary);
		font: inherit;
		letter-spacing: inherit;
		text-transform: inherit;
		white-space: nowrap;
		cursor: pointer;
		opacity: 0.6;
	}

	.stage.static {
		cursor: default;
	}

	.stage:disabled {
		cursor: default;
	}

	.stage.on {
		border-color: var(--onsa-role-active);
		color: var(--onsa-role-active);
		opacity: 1;
	}

	.stage em {
		font-style: normal;
		color: var(--onsa-role-adjustable);
	}

	button.stage:hover:not(:disabled) {
		opacity: 1;
	}

	/* Glow chips (Kaca asap). */
	:global(:root[data-onsa-stage-indicator='glow-chip']) .stage.on {
		text-shadow: 0 0 6px var(--onsa-lit-glow);
	}

	/* LEDs (Deck malam): no border, a lamp before the name. */
	:global(:root[data-onsa-stage-indicator='led']) .stage {
		border-color: transparent;
		padding: 3px 4px;
	}

	:global(:root[data-onsa-stage-indicator='led']) .stage::before {
		content: '';
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--onsa-surface-raised);
		box-shadow: inset 0 1px 1px rgb(0 0 0 / 0.6);
	}

	:global(:root[data-onsa-stage-indicator='led']) .stage.on {
		color: var(--onsa-text-primary);
	}

	:global(:root[data-onsa-stage-indicator='led']) .stage.on::before {
		background: var(--onsa-lit-led);
		box-shadow:
			0 0 6px var(--onsa-lit-led),
			0 0 12px var(--onsa-lit-led);
	}
</style>
