/**
 * The active interface language.
 *
 * It follows the operating system locale until the user picks one in
 * Appearance; the choice is stored by the backend (SPEC section 1).
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

/** Looks up one piece of interface text, filling in `{name}` values. */
export function t(key: MessageKey, values?: Record<string, string | number>): string {
	let text: string = dictionaries[locale][key];
	if (values) {
		for (const [name, value] of Object.entries(values)) {
			text = text.replaceAll(`{${name}}`, String(value));
		}
	}
	return text;
}

export { LOCALES, isLocale, type Locale, type MessageKey } from './dictionary';
