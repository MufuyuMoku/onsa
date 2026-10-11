<!--
	The sky over the window, for a theme that draws one (SPEC M16): soft
	clouds drifting very slowly, a few sparkles, and one small eye.

	All of it is for the look alone: hidden from screen readers, taking no
	pointer, and standing behind everything else. The clouds and sparkles
	move, so they stop in power saving and under reduced motion; the
	sparkles brighten and fade over seconds, far slower than once a second
	(SPEC section 9.1). The eye is still: drawn simply and faintly, it never
	blinks and never follows the pointer.
-->
<script lang="ts">
	import { activeTheme } from '$lib/theme/index.svelte';

	const decor = $derived(activeTheme()?.decor ?? []);
</script>

<div class="sky" aria-hidden="true">
	{#if decor.includes('clouds')}
		<div class="clouds drifts">
			<i style="top: 50px; left: 22%; width: 180px; height: 46px; opacity: 0.9"></i>
			<i style="top: 96px; left: 58%; width: 120px; height: 34px; opacity: 0.75"></i>
			<i style="top: 170px; left: 40%; width: 220px; height: 50px; opacity: 0.6"></i>
		</div>
	{/if}
	{#if decor.includes('sparkles')}
		<div class="sparkles drifts">
			<span style="top: 70px; left: 52%; font-size: 16px">✧</span>
			<span style="top: 230px; left: 24%; font-size: 11px; animation-delay: -2s">✧</span>
			<span style="top: 140px; left: 86%; font-size: 13px; animation-delay: -4s">✧</span>
		</div>
	{/if}
	{#if decor.includes('sky-eye')}
		<svg class="eye" viewBox="0 0 64 30">
			<path d="M2 15 C 18 0, 46 0, 62 15 C 46 30, 18 30, 2 15 Z" />
			<circle class="iris" cx="32" cy="15" r="8" />
			<circle class="pupil" cx="32" cy="15" r="3.5" />
		</svg>
	{/if}
</div>

<style>
	.sky {
		position: absolute;
		inset: 0;
		z-index: -1;
		overflow: hidden;
		pointer-events: none;
		/* The light from above. */
		background: radial-gradient(ellipse at 50% 0%, rgb(255 255 255 / 0.55), transparent 60%);
	}

	.clouds i {
		position: absolute;
		border-radius: 50%;
		background: #fff;
		filter: blur(10px);
		animation: cross 140s linear infinite alternate;
	}

	.clouds i:nth-child(2) {
		animation-duration: 190s;
	}

	.clouds i:nth-child(3) {
		animation-duration: 230s;
	}

	@keyframes cross {
		to {
			transform: translateX(18vw);
		}
	}

	.sparkles span {
		position: absolute;
		color: #fff;
		text-shadow:
			0 0 6px #fff,
			0 0 12px var(--onsa-lit-highlight);
		animation: twinkle 6s ease-in-out infinite;
	}

	/* Brighter and back over six seconds: never a blink. */
	@keyframes twinkle {
		50% {
			opacity: 0.45;
		}
	}

	.eye {
		position: absolute;
		top: 34px;
		right: 18%;
		width: 64px;
		height: 30px;
		opacity: 0.55;
	}

	.eye path {
		fill: #fff;
		stroke: var(--onsa-role-label);
		stroke-width: 1.5;
	}

	.eye .iris {
		fill: var(--onsa-role-position);
	}

	.eye .pupil {
		fill: var(--onsa-text-primary);
	}
</style>
