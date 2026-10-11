/**
 * Ornament text: words a theme carries for its look alone (SPEC section
 * 9.3, layer 7).
 *
 * A theme names keys of the form `ornament.<theme>.<slot>`; the slot says
 * where the words go, and the dictionary says what they are, in both
 * languages. Nothing here knows a theme by name: any theme that names a
 * key for a slot gets words in that slot. Ornament text never stands in
 * for the name of anything that does something, and it is hidden from
 * screen readers wherever it is drawn.
 */

import { textFor } from './i18n/index.svelte';
import { activeTheme } from './theme/index.svelte';

/** Where ornament text can go. */
export type OrnamentSlot = 'sidebar' | 'title' | 'strip' | 'display';

/** The words the theme in use puts in a slot, or nothing. */
export function ornament(slot: OrnamentSlot): string | null {
	const key = activeTheme()?.ornament.find((one) => one.endsWith(`.${slot}`));
	return key ? textFor(key) : null;
}
