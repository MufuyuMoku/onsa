<!-- The sleep timer (SPEC section 3.5), set from the transport. -->
<script lang="ts">
	import type { SleepWhen } from '$lib/backend';
	import { clock } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';
	import { armSleep, cancelSleep, player } from '$lib/player.svelte';
	import Icon from './Icon.svelte';

	const PRESETS = [15, 30, 45, 60];
	const ACTIONS: ['pause' | 'stop' | 'quit', MessageKey][] = [
		['pause', 'sleep.pause'],
		['stop', 'sleep.stop'],
		['quit', 'sleep.quit']
	];

	let open = $state(false);
	let minutes = $state(30);
	let tracks = $state(3);
	let action = $state<'pause' | 'stop' | 'quit'>('pause');
	let fadeSeconds = $state(10);

	const sleep = $derived(player.sleep);
	const left = $derived(
		sleep.secondsLeft === null ? null : clock(Math.max(0, Math.round(sleep.secondsLeft)))
	);

	function start(when: SleepWhen): void {
		armSleep({ when, action, fadeSeconds }).catch(() => {});
		open = false;
	}
</script>

<div class="sleep">
	<button
		type="button"
		class="icon-btn"
		aria-pressed={sleep.armed}
		aria-label={t('sleep.title')}
		title={sleep.armed && left ? t('sleep.left', { time: left }) : t('sleep.title')}
		onclick={() => (open = !open)}
	>
		<Icon name="timer" />
	</button>

	{#if sleep.armed}
		<span class="numeric badge">
			{#if sleep.fading}
				{t('sleep.fading')}
			{:else if sleep.tracksLeft !== null && sleep.tracksLeft > 1}
				{t('sleep.tracksLeft', { n: sleep.tracksLeft })}
			{:else if left}
				{left}
			{/if}
		</span>
	{/if}

	{#if open}
		<div
			class="backdrop"
			role="presentation"
			onclick={() => (open = false)}
		></div>
		<section class="panel">
			<h2 class="label">{t('sleep.title')}</h2>

			<div class="group">
				<span class="label">{t('sleep.when')}</span>
				<div class="chips">
					{#each PRESETS as preset (preset)}
						<button
							type="button"
							class="chip"
							onclick={() => start({ kind: 'minutes', minutes: preset })}
						>
							{t('sleep.minutes', { n: preset })}
						</button>
					{/each}
					<button type="button" class="chip" onclick={() => start({ kind: 'endOfTrack' })}>
						{t('sleep.endOfTrack')}
					</button>
				</div>
			</div>

			<div class="group">
				<label class="field-row">
					<span class="label">{t('sleep.customMinutes')}</span>
					<input class="field numeric" type="number" min="1" max="600" bind:value={minutes} />
					<button
						type="button"
						class="btn"
						onclick={() => start({ kind: 'minutes', minutes: Math.max(1, minutes) })}
					>
						{t('sleep.start')}
					</button>
				</label>
				<label class="field-row">
					<span class="label">{t('sleep.customTracks')}</span>
					<input class="field numeric" type="number" min="1" max="100" bind:value={tracks} />
					<button
						type="button"
						class="btn"
						onclick={() => start({ kind: 'tracks', tracks: Math.max(1, tracks) })}
					>
						{t('sleep.start')}
					</button>
				</label>
			</div>

			<div class="group">
				<span class="label">{t('sleep.action')}</span>
				<div class="chips">
					{#each ACTIONS as [value, label] (value)}
						<button
							type="button"
							class="chip"
							aria-pressed={action === value}
							onclick={() => (action = value)}>{t(label)}</button
						>
					{/each}
				</div>
			</div>

			<label class="field-row">
				<span class="label">{t('sleep.fade')}</span>
				<input type="range" min="0" max="60" step="1" bind:value={fadeSeconds} />
				<span class="numeric value">{fadeSeconds} s</span>
			</label>

			{#if sleep.armed}
				<button type="button" class="btn" onclick={() => cancelSleep().catch(() => {})}>
					{t('sleep.cancel')}
				</button>
			{/if}
		</section>
	{/if}
</div>

<style>
	.sleep {
		position: relative;
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.icon-btn[aria-pressed='true'] {
		border-color: var(--onsa-role-active);
		color: var(--onsa-role-active);
	}

	.badge {
		font-size: 11px;
		color: var(--onsa-role-active);
	}

	.backdrop {
		position: fixed;
		inset: 0;
		z-index: 10;
	}

	.panel {
		position: absolute;
		right: 0;
		bottom: calc(100% + 10px);
		z-index: 11;
		display: grid;
		gap: 12px;
		width: 320px;
		padding: 14px 16px;
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		border-radius: var(--onsa-radius-md);
		background: var(--onsa-surface-body);
		box-shadow: 0 18px 40px rgb(0 0 0 / 0.5);
	}

	h2 {
		margin: 0;
		font-size: 11px;
		font-weight: 500;
	}

	.group {
		display: grid;
		gap: 6px;
	}

	.chips {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.field-row {
		display: grid;
		grid-template-columns: auto minmax(0, 1fr) auto;
		align-items: center;
		gap: 8px;
		font-size: 12.5px;
	}

	.field-row input[type='number'] {
		width: 100%;
	}

	.value {
		min-width: 44px;
		color: var(--onsa-role-adjustable);
		text-align: right;
	}
</style>
