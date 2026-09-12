<!-- Small inline icons; no icon library (SPEC section 1). -->
<script lang="ts">
	type Name = 'play' | 'pause' | 'next' | 'previous' | 'search' | 'close' | 'back' | 'volume';

	let { name }: { name: Name } = $props();

	const PATHS: Record<Name, string> = {
		play: 'M8 5v14l11-7z',
		pause: 'M6 5h4v14H6zM14 5h4v14h-4z',
		next: 'M6 6l8.5 6L6 18zM16 6h2v12h-2z',
		previous: 'M18 6l-8.5 6L18 18zM6 6h2v12H6z',
		search:
			'M10 4a6 6 0 1 0 3.9 10.6l4.8 4.8 1.4-1.4-4.8-4.8A6 6 0 0 0 10 4zm0 2a4 4 0 1 1 0 8 4 4 0 0 1 0-8z',
		close:
			'M6.4 5 5 6.4 10.6 12 5 17.6 6.4 19l5.6-5.6 5.6 5.6 1.4-1.4-5.6-5.6L19 6.4 17.6 5 12 10.6z',
		back: 'M15.4 6 14 4.6 6.6 12l7.4 7.4 1.4-1.4-6-6z',
		volume: 'M4 9v6h4l5 4V5L8 9zM16 8.5v7a4 4 0 0 0 0-7z'
	};
</script>

<svg viewBox="0 0 24 24" aria-hidden="true" focusable="false"><path d={PATHS[name]} /></svg>

<style>
	svg {
		width: 16px;
		height: 16px;
		fill: currentColor;
		display: block;
	}
</style>
