/**
 * What the window knows about the running application, and the start-up
 * sequence that fills it in.
 */

import { appInfo, BackendError, type AppInfo } from '$lib/backend';
import { detectLocale, setLocale } from '$lib/i18n/index.svelte';
import type { MessageKey } from '$lib/i18n/dictionary';
import { loadThemes } from '$lib/theme/index.svelte';

let info = $state<AppInfo | null>(null);
let failure = $state<MessageKey | null>(null);

/** Name, version and starting theme, or `null` until the backend answers. */
export function application(): AppInfo | null {
	return info;
}

/** Why the application could not report itself, if that is where we are. */
export function applicationFailure(): MessageKey | null {
	return failure;
}

/**
 * Brings the window up: the interface language follows the system, then the
 * starting theme is put on.
 */
export async function start(root: HTMLElement): Promise<void> {
	setLocale(detectLocale());
	try {
		info = await appInfo();
	} catch (error) {
		failure = error instanceof BackendError ? error.messageKey : 'error.backend';
		return;
	}
	failure = null;
	await loadThemes(info.defaultThemeId, root);
}
