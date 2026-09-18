<!-- The settings area: one page per section. -->
<script lang="ts">
	import type { SettingsSection } from '$lib/app.svelte';
	import AboutSettings from './AboutSettings.svelte';
	import AppearanceSettings from './AppearanceSettings.svelte';
	import DspSettings from './DspSettings.svelte';
	import LibrarySettings from './LibrarySettings.svelte';
	import LyricsSettings from './LyricsSettings.svelte';
	import MetadataSettings from './MetadataSettings.svelte';
	import OutputSettings from './OutputSettings.svelte';

	let { section }: { section: SettingsSection } = $props();
</script>

<div class="scroll">
	{#if section === 'output'}
		<OutputSettings />
	{:else if section === 'dsp'}
		<DspSettings />
	{:else if section === 'library'}
		<LibrarySettings />
	{:else if section === 'metadata'}
		<MetadataSettings />
	{:else if section === 'lyrics'}
		<LyricsSettings />
	{:else if section === 'appearance'}
		<AppearanceSettings />
	{:else}
		<AboutSettings />
	{/if}
</div>

<style>
	.scroll {
		height: 100%;
		overflow-y: auto;
	}
</style>
