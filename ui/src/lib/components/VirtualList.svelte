<!--
	A list that only renders the rows in view (SPEC section 5.3). Rows are
	fetched a page at a time through `load`, so the interface never holds the
	whole library (SPEC section 13.1). A change of `version` drops the cached
	pages and reloads what is visible.
-->
<script lang="ts" generics="T">
	import type { Snippet } from 'svelte';
	import { SvelteMap } from 'svelte/reactivity';

	interface Props {
		count: number;
		rowHeight: number;
		load: (offset: number, limit: number) => Promise<T[]>;
		row: Snippet<[T | undefined, number]>;
		label: string;
		version?: unknown;
		/** Called as the list is scrolled, with how far down it now is. */
		onmoved?: (top: number) => void;
	}

	let { count, rowHeight, load, row, label, version, onmoved }: Props = $props();

	const PAGE = 100;
	/** Rows rendered beyond the visible ones, for smooth scrolling. */
	const OVERSCAN = 12;

	let viewport: HTMLDivElement | undefined = $state();
	let scrollTop = $state(0);
	let height = $state(0);
	const pages = new SvelteMap<number, T[]>();
	const requested = new Set<number>();
	let generation = 0;

	const first = $derived(Math.max(0, Math.floor(scrollTop / rowHeight) - OVERSCAN));
	const last = $derived(Math.min(count, Math.ceil((scrollTop + height) / rowHeight) + OVERSCAN));
	const indices = $derived(Array.from({ length: Math.max(0, last - first) }, (_, i) => first + i));

	$effect(() => {
		void version;
		void load;
		generation += 1;
		requested.clear();
		pages.clear();
	});

	$effect(() => {
		if (last <= first) return;
		const token = generation;
		for (let page = Math.floor(first / PAGE); page <= Math.floor((last - 1) / PAGE); page++) {
			if (pages.has(page) || requested.has(page)) continue;
			requested.add(page);
			load(page * PAGE, PAGE)
				.then((items) => {
					if (token === generation) pages.set(page, items);
				})
				.catch(() => {
					if (token === generation) requested.delete(page);
				});
		}
	});

	function item(index: number): T | undefined {
		return pages.get(Math.floor(index / PAGE))?.[index % PAGE];
	}

	/** Brings a row into view, putting it in the middle if it is not there. */
	export function reveal(index: number): void {
		if (!viewport) return;
		const top = index * rowHeight;
		if (top < viewport.scrollTop || top + rowHeight > viewport.scrollTop + viewport.clientHeight) {
			scrollTo(Math.max(0, top - viewport.clientHeight / 2));
		}
	}

	/**
	 * Puts the list at a given height, rows and all.
	 *
	 * A row that has never been drawn is still somewhere to scroll to: the
	 * spacer is as tall as the whole list, and which rows to draw is worked
	 * out from the offset rather than from what happens to exist.
	 *
	 * Says whether it took. A viewport whose content is not yet as tall as
	 * the offset asked for is clamped back by the browser, and a caller
	 * restoring a position needs to know that so it can try again rather
	 * than believe the list is where it asked for.
	 */
	export function scrollTo(top: number): boolean {
		if (!viewport) return false;
		viewport.scrollTop = top;
		scrollTop = viewport.scrollTop;
		onmoved?.(scrollTop);
		return Math.abs(scrollTop - top) < 1;
	}
</script>

<div
	class="viewport"
	role="list"
	aria-label={label}
	bind:this={viewport}
	bind:clientHeight={height}
	onscroll={() => {
		scrollTop = viewport?.scrollTop ?? 0;
		onmoved?.(scrollTop);
	}}
>
	<div class="spacer" style:height="{count * rowHeight}px">
		{#each indices as index (index)}
			<div
				class="slot"
				role="listitem"
				style:transform="translateY({index * rowHeight}px)"
				style:height="{rowHeight}px"
			>
				{@render row(item(index), index)}
			</div>
		{/each}
	</div>
</div>

<style>
	.viewport {
		position: relative;
		height: 100%;
		overflow-y: auto;
		overflow-x: hidden;
		contain: strict;
	}

	.spacer {
		position: relative;
		width: 100%;
	}

	.slot {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
	}
</style>
