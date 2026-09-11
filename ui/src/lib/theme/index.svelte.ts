/**
 * The theme the window is wearing.
 *
 * M0 reads the built-in themes through the backend and dresses the window in
 * one of them. Remembering the choice, user themes and live preview arrive
 * with the settings screen in M4.
 */

import { themeList, BackendError } from '$lib/backend';
import type { MessageKey } from '$lib/i18n/dictionary';
import { applyTheme } from './apply';
import type { Theme } from './types';

let themes = $state<Theme[]>([]);
let active = $state<Theme | null>(null);
let failure = $state<MessageKey | null>(null);

/** Every theme the user can pick. */
export function availableThemes(): readonly Theme[] {
	return themes;
}

/** The theme currently applied, or `null` while it is still loading. */
export function activeTheme(): Theme | null {
	return active;
}

/** Why no theme could be loaded, if that is where we are. */
export function themeFailure(): MessageKey | null {
	return failure;
}

/** Loads the themes and puts on the one the application starts with. */
export async function loadThemes(startId: string, root: HTMLElement): Promise<void> {
	try {
		themes = await themeList();
	} catch (error) {
		failure = error instanceof BackendError ? error.messageKey : 'error.backend';
		return;
	}

	const start = themes.find((theme) => theme.id === startId) ?? themes[0];
	if (!start) {
		failure = 'error.theme_not_found';
		return;
	}
	failure = null;
	wear(start, root);
}

/** Switches to another theme that is already loaded. */
export function selectTheme(id: string, root: HTMLElement): void {
	const theme = themes.find((candidate) => candidate.id === id);
	if (!theme) {
		failure = 'error.theme_not_found';
		return;
	}
	wear(theme, root);
}

function wear(theme: Theme, root: HTMLElement): void {
	active = theme;
	applyTheme(theme, root);
}
