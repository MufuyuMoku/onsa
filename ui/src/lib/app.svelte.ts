/**
 * What the window knows about the running application, where it is, and the
 * start-up sequence that fills it in.
 */

import {
	appInfo,
	appState,
	failureKey,
	setLocaleSetting,
	type AppInfo,
	type AppState
} from '$lib/backend';
import { detectLocale, isLocale, setLocale, type Locale } from '$lib/i18n/index.svelte';
import type { MessageKey } from '$lib/i18n/dictionary';
import { initLibrary } from '$lib/library.svelte';
import { initPlayer } from '$lib/player.svelte';
import { loadSettings } from '$lib/settings.svelte';
import { loadThemes } from '$lib/theme/index.svelte';

export type SettingsSection = 'output' | 'dsp' | 'library' | 'appearance' | 'about';

/** What the main area shows. */
export type View =
	| { kind: 'tracks' }
	| { kind: 'albums' }
	| { kind: 'album'; id: number }
	| { kind: 'settings'; section: SettingsSection };

let info = $state<AppInfo | null>(null);
let stored = $state<AppState | null>(null);
let failure = $state<MessageKey | null>(null);
let ready = $state(false);
let firstRun = $state(false);
let view = $state<View>({ kind: 'tracks' });

export const app = {
	/** Name, version and starting theme, once the backend answered. */
	get info() {
		return info;
	},
	/** What was restored at start. */
	get state() {
		return stored;
	},
	/** Why the window could not start, if that is where we are. */
	get failure() {
		return failure;
	},
	/** Whether everything is loaded. */
	get ready() {
		return ready;
	},
	/** Whether the first screen shows (no library folder yet). */
	get firstRun() {
		return firstRun;
	},
	/** What the main area shows. */
	get view() {
		return view;
	}
};

/**
 * Brings the window up: language, theme, then the player, library and
 * settings the rest of the interface reads.
 */
export async function start(root: HTMLElement): Promise<void> {
	setLocale(detectLocale());
	try {
		info = await appInfo();
		stored = await appState();
		if (stored.locale && isLocale(stored.locale)) setLocale(stored.locale);
		await loadThemes(stored.themeId ?? info.defaultThemeId, root);
		firstRun = !stored.hasFolders;
		await Promise.all([initPlayer(), initLibrary(), loadSettings()]);
		failure = null;
		ready = true;
	} catch (error) {
		failure = failureKey(error);
	}
}

/** Switches the interface language and remembers it. */
export function chooseLocale(next: Locale): void {
	setLocale(next);
	setLocaleSetting(next).catch(() => {});
}

/** Moves the main area. */
export function navigate(next: View): void {
	view = next;
}

/** Leaves the first screen for the library. */
export function finishFirstRun(): void {
	firstRun = false;
	view = { kind: 'tracks' };
}
