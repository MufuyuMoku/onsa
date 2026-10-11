<!--
	A ceiling over the window (SPEC M16, chrome `corridor`): a row of
	fluorescent lamps, and the play time stamped at both ends the way a
	camcorder stamps its tape.

	Everything here is for the look alone and hidden from screen readers;
	the time it shows is the real position, which the transport already
	says out loud. One lamp dims slowly now and then — never more than
	once in twenty seconds, never a flash — and stops in power saving and
	under reduced motion (SPEC section 9.1).
-->
<script lang="ts">
	import { t } from '$lib/i18n/index.svelte';
	import { player } from '$lib/player.svelte';

	const playing = $derived(player.snapshot?.state === 'playing');

	/** The position as a camcorder writes it: hours, minutes, seconds. */
	const stamp = $derived.by(() => {
		const whole = Math.max(0, Math.floor(player.position));
		const two = (n: number) => String(n).padStart(2, '0');
		return `${two(Math.floor(whole / 3600))}:${two(Math.floor(whole / 60) % 60)}:${two(whole % 60)}`;
	});
</script>

<div class="ceiling" aria-hidden="true">
	<div class="lamps drifts">
		<i></i><i class="dims"></i><i></i><i></i>
	</div>
	<!-- Under the ceiling, on the wall, where a camcorder's stamp would be. -->
	<div class="stamps">
		<span class="stamp">{playing ? `▶ ${t('camcorder.play')}` : `❚❚ ${t('camcorder.pause')}`}&nbsp;&nbsp;SP</span>
		<span class="stamp numeric">{stamp}</span>
	</div>
</div>

<style>
	.ceiling {
		height: 56px;
		pointer-events: none;
	}

	.lamps {
		display: flex;
		align-items: center;
		justify-content: space-around;
		gap: 60px;
		min-width: 0;
		height: 30px;
		padding: 0 70px;
		border-bottom: var(--onsa-hairline) solid var(--onsa-edge-dark);
		background: var(--onsa-surface-raised);
	}

	.stamps {
		display: flex;
		justify-content: space-between;
		padding: 2px 22px 0;
	}

	.lamps i {
		flex: 1;
		max-width: 160px;
		height: 12px;
		border: 1px solid color-mix(in srgb, var(--onsa-edge-dark) 30%, transparent);
		background: var(--onsa-lit-highlight);
		box-shadow: 0 0 24px 8px color-mix(in srgb, var(--onsa-lit-highlight) 55%, transparent);
	}

	/* Dims slowly for a little over a second, once in twenty-four. */
	.lamps i.dims {
		animation: dim 24s ease-in-out infinite;
	}

	@keyframes dim {
		0%,
		82% {
			opacity: 1;
		}
		88% {
			opacity: 0.55;
		}
		94%,
		100% {
			opacity: 1;
		}
	}

	.stamp {
		font-family: var(--onsa-font-accent);
		font-size: 20px;
		line-height: 1;
		color: var(--onsa-lit-highlight);
		text-shadow: 1px 1px 0 var(--onsa-edge-dark);
		white-space: nowrap;
	}
</style>
