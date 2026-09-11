/**
 * Interface text (SPEC section 9.6).
 *
 * No user visible string is ever written inside a component. Indonesian is
 * the reference dictionary; English has to answer every key it holds.
 */

export const LOCALES = ['id', 'en'] as const;

export type Locale = (typeof LOCALES)[number];

const id = {
	'shell.milestone': 'M0 · kerangka proyek',
	'shell.emptyPanel':
		'Panel masih kosong. Mesin audio, library, dan tampilan lengkap menyusul mulai M1.',
	'shell.loading': 'Memuat…',
	'theme.label': 'Tema',
	'language.label': 'Bahasa',
	'language.id': 'Indonesia',
	'language.en': 'Inggris',
	'error.backend': 'Backend tidak terjangkau. Jalankan Onsa lewat jendela aplikasinya.',
	'error.theme_not_found': 'Tema itu tidak ada.'
} as const;

export type MessageKey = keyof typeof id;

const en: Record<MessageKey, string> = {
	'shell.milestone': 'M0 · project skeleton',
	'shell.emptyPanel':
		'The panel is still empty. The audio engine, the library and the full interface arrive from M1 on.',
	'shell.loading': 'Loading…',
	'theme.label': 'Theme',
	'language.label': 'Language',
	'language.id': 'Indonesian',
	'language.en': 'English',
	'error.backend': 'The backend cannot be reached. Start Onsa from its own window.',
	'error.theme_not_found': 'There is no such theme.'
};

export const dictionaries: Record<Locale, Record<MessageKey, string>> = { id, en };

/** Whether a string names a locale Onsa speaks. */
export function isLocale(value: string): value is Locale {
	return (LOCALES as readonly string[]).includes(value);
}
