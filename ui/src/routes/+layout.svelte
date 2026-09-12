<script lang="ts">
	import '../fonts.css';
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { start } from '$lib/app.svelte';

	let { children } = $props();

	// The window follows the operating system language and puts on the
	// starting theme as soon as the backend answers.
	$effect(() => {
		void start(document.documentElement);
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children()}
