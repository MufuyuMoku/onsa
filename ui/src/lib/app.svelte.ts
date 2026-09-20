/**
 * What the window knows about the running application, where it is, and the
 * start-up sequence that fills it in.
 */

import {
	appInfo,
	appState,
	EVENTS,
	failureKey,
	on,
	setLocaleSetting,
	setMini,
	startupTrouble,
	traySetup,
	type AppInfo,
	type AppState,
	type StartupTrouble
} from '$lib/backend';
import { detectLocale, isLocale, setLocale, t, type Locale } from '$lib/i18n/index.svelte';
import { forgetHelpTopic } from '$lib/layout.svelte';
import type { MessageKey } from '$lib/i18n/dictionary';
import { initLibrary } from '$lib/library.svelte';
import { initPlayer } from '$lib/player.svelte';
import { initPlaylists } from '$lib/playlists.svelte';
import { loadSettings } from '$lib/settings.svelte';
import { loadThemes } from '$lib/theme/index.svelte';

export type SettingsSection =
	| 'output'
	| 'dsp'
	| 'library'
	| 'metadata'
	| 'lyrics'
	| 'appearance'
	| 'about';

/** What the main area shows. */
export type View =
	| { kind: 'tracks' }
	| { kind: 'albums' }
	| { kind: 'album'; id: number }
	| { kind: 'artists' }
	| { kind: 'artist'; name: string }
	| { kind: 'genres' }
	| { kind: 'genre'; name: string }
	| { kind: 'folders' }
	| { kind: 'folder'; path: string }
	| { kind: 'tidy' }
	| { kind: 'downloads' }
	| { kind: 'playlists' }
	| { kind: 'playlist'; id: number }
	| { kind: 'nowPlaying' }
	| { kind: 'help' }
	| { kind: 'settings'; section: SettingsSection };

let info = $state<AppInfo | null>(null);
let stored = $state<AppState | null>(null);
let failure = $state<MessageKey | null>(null);
let trouble = $state<StartupTrouble | null>(null);
let ready = $state(false);
let firstRun = $state(false);
let revisit = $state(false);
let before = $state<View | null>(null);
let mini = $state(false);
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
	/**
	 * Set when the library itself could not be opened.
	 *
	 * Nothing else in the window works in that state — every other command
	 * wants the library — so this is asked first and answered on a screen
	 * of its own.
	 */
	get trouble() {
		return trouble;
	},
	/** Whether everything is loaded. */
	get ready() {
		return ready;
	},
	/** Whether the first screen shows (no library folder yet). */
	get firstRun() {
		return firstRun;
	},
	/**
	 * Whether the first screen was asked for again from the help page,
	 * rather than shown because there is no library yet. The steps are the
	 * same; what differs is that nothing here is owed any more.
	 */
	get revisit() {
		return revisit;
	},
	/** What the main area shows. */
	get view() {
		return view;
	},
	/** Whether the window is the mini player (SPEC section 9.2). */
	get mini() {
		return mini;
	}
};

/**
 * Brings the window up: language, theme, then the player, library and
 * settings the rest of the interface reads.
 */
export async function start(root: HTMLElement): Promise<void> {
	setLocale(detectLocale());
	try {
		trouble = await startupTrouble();
		if (trouble) {
			ready = true;
			return;
		}
		info = await appInfo();
		stored = await appState();
		if (stored.locale && isLocale(stored.locale)) setLocale(stored.locale);
		await loadThemes(stored.themeId ?? info.defaultThemeId, root);
		firstRun = !stored.hasFolders;
		mini = stored.mini;
		// The window decides its own mode; the interface follows it.
		await on<boolean>(EVENTS.windowMode, (next) => (mini = next));
		await Promise.all([initPlayer(), initLibrary(), initPlaylists(), loadSettings()]);
		await refreshTray();
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
	// The tray menu speaks the interface's language too (SPEC section 9.6).
	void refreshTray();
}

/** Hands the tray menu its words. */
export async function refreshTray(): Promise<void> {
	try {
		await traySetup({
			show: t('tray.show'),
			play: t('tray.play'),
			previous: t('transport.previous'),
			next: t('transport.next'),
			quit: t('tray.quit')
		});
	} catch {
		// A tray icon the system refuses is not worth stopping for.
	}
}

/**
 * Asks for the full window or the mini player. The window mode lives in the
 * backend; this only asks, and the answer (or the event, when the change
 * came from elsewhere) is what the interface shows (SPEC section 9.2).
 */
export async function setMiniPlayer(next: boolean): Promise<void> {
	try {
		mini = await setMini(next);
	} catch (error) {
		failure = failureKey(error);
	}
}

/** Moves the main area. */
export function navigate(next: View): void {
	view = next;
	// Help asked for by name belongs to where it was asked from.
	forgetHelpTopic();
}

/** Leaves the first screen for the library, or for a page it pointed at. */
export function finishFirstRun(next?: View): void {
	firstRun = false;
	// Somebody who opened it from the help page is put back where they were.
	view = next ?? (revisit && before ? before : { kind: 'tracks' });
	revisit = false;
	before = null;
}

/** Shows the first screen again, from the help page (SPEC section 9.2). */
export function showFirstRun(): void {
	before = view;
	revisit = true;
	firstRun = true;
}
