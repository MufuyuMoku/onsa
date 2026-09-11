/**
 * The active interface language.
 *
 * It follows the operating system locale on first start (SPEC section 1); the
 * setting that lets the user override it arrives with the settings screen in
 * M4.
 */

import { browser } from '$app/environment';
import { dictionaries, isLocale, type Locale, type MessageKey } from './dictionary';

/** Falls back to Indonesian, the reference dictionary. */
const FALLBACK: Locale = 'id';

let locale = $state<Locale>(FALLBACK);

/** Reads the locale of the surrounding system, as the browser reports it. */
export function detectLocale(): Locale {
	if (!browser) return FALLBACK;
	for (const tag of navigator.languages ?? [navigator.language]) {
		const language = tag.split('-')[0]?.toLowerCase() ?? '';
		if (isLocale(language)) return language;
	}
	return FALLBACK;
}

/** The language the interface is speaking. */
export function currentLocale(): Locale {
	return locale;
}

/** Switches the interface language. */
export function setLocale(next: Locale): void {
	locale = next;
	if (browser) document.documentElement.lang = next;
}

/** Looks up one piece of interface text. */
export function t(key: MessageKey): string {
	return dictionaries[locale][key];
}

export { LOCALES, type Locale, type MessageKey } from './dictionary';
