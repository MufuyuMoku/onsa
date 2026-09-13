<!--
	A panel over the window, for the few things that have to be answered
	before anything else happens: naming a playlist, editing smart rules.

	It shrinks with the window and scrolls inside itself, so it stays whole
	at the smallest window Onsa allows (SPEC section 9.2).
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import { t } from '$lib/i18n/index.svelte';
	import Icon from './Icon.svelte';

	interface Props {
		title: string;
		/** Called when the listener closes without deciding. */
		onclose: () => void;
		/** Wide enough for the rule editor; the default suits a name. */
		wide?: boolean;
		children: Snippet;
		footer?: Snippet;
	}

	const { title, onclose, wide = false, children, footer }: Props = $props();

	let panel: HTMLElement | undefined = $state();

	$effect(() => {
		// The first field is where the listener is going anyway.
		panel?.querySelector<HTMLElement>('input, select, textarea, button')?.focus();
	});
</script>

<svelte:window
	onkeydown={(event) => {
		if (event.key === 'Escape') onclose();
	}}
/>

<!-- The backdrop closes the panel; it is not a control of its own. -->
<div class="backdrop" role="presentation" onclick={onclose}></div>
<div class="wrap" role="presentation">
	<div bind:this={panel} class="panel" class:wide role="dialog" aria-modal="true" aria-label={title}>
		<header>
			<h2 class="ellipsis">{title}</h2>
			<button type="button" class="icon-btn" aria-label={t('playlist.cancel')} onclick={onclose}>
				<Icon name="close" />
			</button>
		</header>
		<div class="content">{@render children()}</div>
		{#if footer}
			<footer>{@render footer()}</footer>
		{/if}
	</div>
</div>

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		z-index: 40;
		background: rgb(0 0 0 / 0.5);
	}

	.wrap {
		position: fixed;
		inset: 0;
		z-index: 41;
		display: grid;
		place-items: center;
		padding: 16px;
		pointer-events: none;
	}

	.panel {
		display: flex;
		flex-direction: column;
		width: min(420px, 100%);
		max-height: 100%;
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		border-radius: var(--onsa-radius-md);
		background: var(--onsa-surface-body);
		box-shadow: 0 18px 40px rgb(0 0 0 / 0.5);
		pointer-events: auto;
	}

	.panel.wide {
		width: min(760px, 100%);
	}

	header {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 12px 12px 12px 16px;
		border-bottom: var(--onsa-hairline) solid var(--onsa-surface-line);
	}

	h2 {
		flex: 1;
		min-width: 0;
		margin: 0;
		font-size: 14px;
		font-weight: 500;
		color: var(--onsa-text-primary);
	}

	.content {
		min-height: 0;
		padding: 16px;
		overflow-y: auto;
		scrollbar-width: thin;
	}

	footer {
		display: flex;
		flex-wrap: wrap;
		justify-content: flex-end;
		gap: 8px;
		padding: 12px 16px;
		border-top: var(--onsa-hairline) solid var(--onsa-surface-line);
	}
</style>
