<!-- Small inline icons; no icon library (SPEC section 1). -->
<script lang="ts">
	type Name =
		| 'play'
		| 'pause'
		| 'next'
		| 'previous'
		| 'search'
		| 'close'
		| 'back'
		| 'volume'
		| 'timer'
		| 'disc'
		| 'mini'
		| 'full'
		| 'menu'
		| 'queue'
		| 'more'
		| 'note'
		| 'artist'
		| 'genre'
		| 'folder'
		| 'sliders'
		| 'eq'
		| 'shelf'
		| 'palette'
		| 'info';

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
		volume: 'M4 9v6h4l5 4V5L8 9zM16 8.5v7a4 4 0 0 0 0-7z',
		timer:
			'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zm0 2a7 7 0 1 1 0 14 7 7 0 0 1 0-14zm1 3h-2v5.4l4.3 2.5 1-1.7-3.3-1.9z',
		disc: 'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zm0 7a2 2 0 1 1 0 4 2 2 0 0 1 0-4z',
		mini: 'M4 14h6v6H8v-2.6L4.6 20.8 3.2 19.4 6.6 16H4zM20 10h-6V4h2v2.6l3.4-3.4 1.4 1.4L17.4 8H20z',
		full: 'M4 4h6v2H7.4l3.4 3.4-1.4 1.4L6 7.4V10H4zM14 14h6v6h-2v-2.6l-3.4 3.4-1.4-1.4 3.4-3.4H14z',
		menu: 'M4 6h16v2H4zM4 11h16v2H4zM4 16h16v2H4z',
		queue: 'M4 6h11v2H4zM4 11h11v2H4zM4 16h8v2H4zM17 11l5 3.5-5 3.5z',
		more: 'M6 10a2 2 0 1 1 0 4 2 2 0 0 1 0-4zm6 0a2 2 0 1 1 0 4 2 2 0 0 1 0-4zm6 0a2 2 0 1 1 0 4 2 2 0 0 1 0-4z',
		note: 'M9 18a3 3 0 1 1-2-2.8V6l11-2v10.2A3 3 0 1 1 16 14V8.5L9 10z',
		artist:
			'M12 4a4 4 0 1 1 0 8 4 4 0 0 1 0-8zm0 10c4.4 0 8 2.2 8 4.5V20H4v-1.5C4 16.2 7.6 14 12 14z',
		genre: 'M4 4h7.2l8.8 8.8-7.2 7.2L4 11.2zm3.4 2.6a1.8 1.8 0 1 0 0 3.6 1.8 1.8 0 0 0 0-3.6z',
		folder: 'M3 5h6l2 2.4h10V19H3zm2 2v10h14V9.4H10L8 7z',
		sliders:
			'M4 7h9v2H4zm11 0h5v2h-5zM4 15h5v2H4zm7 0h9v2h-9zM13 5h2v6h-2zM9 13h2v6H9z',
		eq: 'M4 18h2V9H4zm4.5 0h2V4h-2zM13 18h2v-6h-2zm4.5 0h2V7h-2z',
		shelf: 'M4 4h16v5H4zm0 7h16v5H4zm2 7h12v2H6zM6 6v1h2V6zm0 7v1h2v-1z',
		palette:
			'M12 3a9 9 0 0 0 0 18c1.3 0 2-.8 2-1.7 0-.5-.2-.9-.5-1.2-.3-.3-.5-.7-.5-1.1 0-1 .8-1.8 1.8-1.8H16a5 5 0 0 0 5-5c0-4-4-7.2-9-7.2zM7.5 12a1.5 1.5 0 1 1 0-3 1.5 1.5 0 0 1 0 3zm3.5-4a1.5 1.5 0 1 1 0-3 1.5 1.5 0 0 1 0 3zm5 0a1.5 1.5 0 1 1 0-3 1.5 1.5 0 0 1 0 3z',
		info: 'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zm-1 4h2v2h-2zm0 4h2v6h-2z'
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
