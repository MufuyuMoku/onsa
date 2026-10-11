/**
 * What goes where in the window: the layout presets of M16.
 *
 * Three standard presets every theme can be worn with, and a theme's own
 * signature layout beside them. All four are values of one shape,
 * `Arrangement`, so the window reads a value rather than asking which
 * theme it is wearing. That is the rule: a signature layout is data, not
 * a branch that names a theme (SPEC section 9.3, layer 8).
 *
 * This module knows nothing about state or storage, so it can be read and
 * tested on its own; `arrangement.svelte.ts` is the thin layer that reads
 * the stored choice and the theme in use.
 */

import type { Arrangement, Theme } from './theme/types';

/** The standard presets, in the order the picker shows them. */
export const PRESETS = ['classic', 'gallery', 'fullscreen'] as const;

/** One of them. */
export type Preset = (typeof PRESETS)[number];

/** What can be chosen: a standard preset, or the theme's own layout. */
export type LayoutChoice = Preset | 'signature';

/**
 * Klasik: the window Onsa has always drawn (SPEC section 9.2) — sidebar,
 * content, the queue beside it when there is room, the full transport, and
 * the signal path strip.
 */
export const CLASSIC: Arrangement = {
	sidebar: 'shown',
	queue: 'docked',
	transport: 'full',
	signal: true,
	libraryHome: 'tracks',
	nowPlaying: 'classic',
	cover: 'plain',
	position: 'bar',
	chrome: 'none',
	plates: false,
	panels: {}
};

/**
 * Galeri: sidebar, a grid of covers for the library, and one slim player
 * bar at the bottom. The queue opens over the content rather than taking a
 * column, and the signal strip stays out of the way.
 */
export const GALLERY: Arrangement = {
	...CLASSIC,
	queue: 'floating',
	transport: 'bar',
	signal: false,
	libraryHome: 'albums'
};

/**
 * Layar penuh: the cover or the picture large, with the words, and only
 * the controls playing needs. Navigation waits behind a button.
 */
export const FULLSCREEN: Arrangement = {
	...CLASSIC,
	sidebar: 'hidden',
	queue: 'floating',
	transport: 'minimal',
	signal: false,
	nowPlaying: 'immersive',
	panels: { cover: true, lyrics: true, details: false, visualizer: false }
};

const STANDARD: Record<Preset, Arrangement> = {
	classic: CLASSIC,
	gallery: GALLERY,
	fullscreen: FULLSCREEN
};

/** Whether a stored word is one of the choices. */
export function isChoice(word: unknown): word is LayoutChoice {
	return word === 'signature' || (PRESETS as readonly unknown[]).includes(word);
}

/**
 * The choice in force.
 *
 * Nobody choosing means the theme's own layout when it has one, and Klasik
 * otherwise — so picking a theme brings its layout with it (SPEC M16). A
 * stored `signature` on a theme without one is Klasik too: a user theme
 * from before M16 has no layout of its own, and the window must still
 * have an arrangement.
 */
export function effectiveChoice(stored: string | null | undefined, theme: Theme | null): LayoutChoice {
	const hasOwn = theme?.signature != null;
	if (stored === 'signature' || !isChoice(stored)) return hasOwn ? 'signature' : 'classic';
	return stored;
}

/** The arrangement a choice stands for, with this theme. */
export function arrangementFor(stored: string | null | undefined, theme: Theme | null): Arrangement {
	const choice = effectiveChoice(stored, theme);
	if (choice === 'signature' && theme?.signature) return theme.signature.arrangement;
	return STANDARD[choice === 'signature' ? 'classic' : choice];
}

/**
 * What a theme's layout looks like when the theme is worn in a standard
 * preset: none of it. The theme keeps its look — colours, fonts, material
 * — but the arrangement is the preset's (SPEC M16). This is here so that
 * statement is something a test can hold.
 */
export const standardArrangement = (preset: Preset): Arrangement => STANDARD[preset];
