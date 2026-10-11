/**
 * The layout in force, as the interface holds it (SPEC M16).
 *
 * The rule lives in `arrangement.ts`; this reads the stored choice and the
 * theme being worn, and writes a new choice back.
 */

import { app, navigate } from './app.svelte';
import { arrangementFor, effectiveChoice, type LayoutChoice } from './arrangement';
import { settings, updateDisplay } from './settings.svelte';
import { activeTheme, selectTheme } from './theme/index.svelte';
import type { Arrangement } from './theme/types';

export { PRESETS, type LayoutChoice, type Preset } from './arrangement';

/** The arrangement the window is drawn by right now. */
export function arrangement(): Arrangement {
	return arrangementFor(settings.value?.display.layout, activeTheme());
}

/** Which choice that is. */
export function layoutChoice(): LayoutChoice {
	return effectiveChoice(settings.value?.display.layout, activeTheme());
}

/**
 * Arranges the window by a choice, and remembers it.
 *
 * Which Now Playing panels show is the layout's own business: each layout
 * keeps its own choices (see `panels.svelte.ts`), so nothing is written
 * here. The window goes where the layout starts: the cover grid for
 * Galeri, the picture for Layar penuh.
 */
export function chooseLayout(choice: LayoutChoice): void {
	void updateDisplay({ layout: choice });
	settle();
}

/**
 * Wears a theme, and lets its own layout come with it (SPEC M16): the
 * stored choice is cleared, so a theme with a layout of its own is worn in
 * it, and one without is worn in Klasik. Somebody who wants a standard
 * preset with it picks one afterwards.
 */
export function wearTheme(id: string, root: HTMLElement): void {
	selectTheme(id, root);
	void updateDisplay({ layout: null });
	settle();
}

/** Puts the panels and the page where the layout in force says. */
function settle(): void {
	const now = arrangement();
	if (now.nowPlaying === 'immersive') {
		navigate({ kind: 'nowPlaying' });
	} else if (now.libraryHome === 'albums' && app.view.kind === 'tracks') {
		navigate({ kind: 'albums' });
	}
}
