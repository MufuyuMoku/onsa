<!--
	The signal path strip (SPEC section 9.2): the chain the audio really goes
	through. Click a stage to switch it on or off; right-click to open its
	settings.

	It is always one line. As the window narrows the stages drop their
	readings, then their long names, and finally the ones furthest from the
	listener's ear fold into a button that opens them in place — nothing is
	lost, only put away (SPEC section 9.2).
-->
<script lang="ts">
	import type { RgMode } from '$lib/backend';
	import { navigate, type SettingsSection } from '$lib/app.svelte';
	import { db, rate } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import { tip } from '$lib/tip';
	import { measure } from '$lib/layout.svelte';
	import { player } from '$lib/player.svelte';
	import { settings, updateDsp } from '$lib/settings.svelte';
	import Icon from './Icon.svelte';

	const signal = $derived(player.snapshot?.signal ?? null);
	const dsp = $derived(settings.value?.dsp ?? null);

	/** Width of the strip itself. */
	let width = $state(1200);
	/** Readings beside each stage's name. */
	const detailed = $derived(width >= 1060);
	/** Names in full rather than their short forms. */
	const spelled = $derived(width >= 900);
	/** ReplayGain and the limiter move into the button below this. */
	const folded = $derived(width < 780);

	let open = $state(false);
	$effect(() => {
		if (!folded) open = false;
	});

	/** The mode ReplayGain returns to when switched back on. */
	let lastMode = $state<Exclude<RgMode, 'off'>>('auto');
	$effect(() => {
		if (dsp && dsp.replaygainMode !== 'off') lastMode = dsp.replaygainMode;
	});

	function show(section: SettingsSection, event: MouseEvent): void {
		event.preventDefault();
		open = false;
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

<div
	class="path"
	role="group"
	aria-label={t('signal.label')}
	use:tip={t('signal.hint')}
	use:measure={(seen) => (width = seen)}
>
	<span class="stage static source" class:on={source !== null}>{source ?? t('signal.idle')}</span>

	{#if signal?.resample}
		<span class="sep"></span>
		<button
			type="button"
			class="stage on"
			onclick={(event) => show('output', event)}
			oncontextmenu={(event) => show('output', event)}
		>
			{spelled ? t('signal.resample') : t('signal.resampleShort')}{#if detailed}<em
					>{rate(signal.resample.to)}</em
				>{/if}
		</button>
	{/if}

	{#if !folded}
		<span class="sep"></span>
		<button
			type="button"
			class="stage"
			class:on={dsp?.replaygainMode !== 'off'}
			aria-pressed={dsp?.replaygainMode !== 'off'}
			disabled={!dsp}
			onclick={() =>
				updateDsp({ replaygainMode: dsp?.replaygainMode === 'off' ? lastMode : 'off' })}
			oncontextmenu={(event) => show('output', event)}
		>
			{spelled ? t('signal.replaygain') : t('signal.replaygainShort')}{#if detailed && rgDetail}<em
					>{rgDetail}</em
				>{/if}
		</button>
	{/if}

	<span class="sep"></span>
	<button
		type="button"
		class="stage"
		class:on={dsp?.eqEnabled}
		aria-pressed={dsp?.eqEnabled ?? false}
		disabled={!dsp}
		onclick={() => updateDsp({ eqEnabled: !dsp?.eqEnabled })}
		oncontextmenu={(event) => show('dsp', event)}
	>
		{t('signal.eq')}{#if detailed && eqDetail}<em>{eqDetail}</em>{/if}
	</button>

	{#if !folded}
		<span class="sep"></span>
		<button
			type="button"
			class="stage"
			class:on={dsp?.limiterEnabled}
			aria-pressed={dsp?.limiterEnabled ?? false}
			disabled={!dsp}
			onclick={() => updateDsp({ limiterEnabled: !dsp?.limiterEnabled })}
			oncontextmenu={(event) => show('dsp', event)}
		>
			{t('signal.limiter')}
		</button>
	{/if}

	<span class="sep"></span>
	<button
		type="button"
		class="stage output"
		class:on={signal?.output != null}
		onclick={(event) => show('output', event)}
		oncontextmenu={(event) => show('output', event)}
	>
		{#if signal?.output}
			{signal.output.backend}{#if detailed}<em
					>{signal.output.name} · {rate(signal.output.sampleRate)}</em
				>{/if}
		{:else}
			{t('output.none')}
		{/if}
	</button>

	{#if folded}
		<button
			type="button"
			class="stage fold"
			aria-label={t('signal.more')}
			aria-expanded={open}
			title={t('signal.more')}
			onclick={() => (open = !open)}
		>
			<Icon name="more" />
		</button>
	{/if}
</div>

{#if folded && open}
	<!-- Closing the panel is what the backdrop is for; it is not a control. -->
	<div class="backdrop" role="presentation" onclick={() => (open = false)}></div>
	<div class="folded" role="group" aria-label={t('signal.label')}>
		<button
			type="button"
			class="stage"
			class:on={dsp?.replaygainMode !== 'off'}
			aria-pressed={dsp?.replaygainMode !== 'off'}
			disabled={!dsp}
			onclick={() => updateDsp({ replaygainMode: dsp?.replaygainMode === 'off' ? lastMode : 'off' })}
		>
			{t('signal.replaygain')}{#if rgDetail}<em>{rgDetail}</em>{/if}
		</button>
		<button
			type="button"
			class="stage"
			class:on={dsp?.limiterEnabled}
			aria-pressed={dsp?.limiterEnabled ?? false}
			disabled={!dsp}
			onclick={() => updateDsp({ limiterEnabled: !dsp?.limiterEnabled })}
		>
			{t('signal.limiter')}
		</button>
	</div>
{/if}

<style>
	.path {
		display: flex;
		flex-wrap: nowrap;
		align-items: center;
		gap: 8px;
		min-width: 0;
		overflow: hidden;
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
		flex: none;
		width: 14px;
		height: var(--onsa-hairline);
		background: var(--onsa-role-label);
		opacity: 0.5;
	}

	/* The two stages that carry a name of their own give up room first. */
	.stage.source,
	.stage.output {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		display: block;
		line-height: 1.6;
	}

	.stage.output {
		flex: 0 1 auto;
	}

	.fold {
		flex: none;
		padding: 2px 6px;
	}

	.fold :global(svg) {
		display: block;
		width: 16px;
		height: 16px;
		fill: currentColor;
	}

	.backdrop {
		position: fixed;
		inset: 0;
		z-index: 10;
	}

	.folded {
		position: fixed;
		right: 12px;
		bottom: 44px;
		z-index: 11;
		display: grid;
		gap: 6px;
		padding: 8px;
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-raised);
		box-shadow: 0 10px 24px rgb(0 0 0 / 0.45);
		font-family: var(--onsa-font-label);
		font-size: 12px;
		color: var(--onsa-role-label);
	}

	:global(:root[data-onsa-label-case='uppercase']) .folded {
		text-transform: uppercase;
		letter-spacing: 0.06em;
	}

	.stage {
		flex: none;
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
