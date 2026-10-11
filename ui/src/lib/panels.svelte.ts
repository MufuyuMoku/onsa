/**
 * The panel state as the interface holds it (SPEC section 15, M14).
 *
 * The rule itself lives in `panels.ts`, which knows nothing about state or
 * storage and can be read and tested on its own. This is only the thin
 * layer that reads it out of the display settings and writes a change back.
 *
 * Each layout keeps its own choices (SPEC M16). Klasik reads and writes the
 * map M14 has always stored, so nothing anybody decided before is lost;
 * every other layout has a map of its own, and starts from what that layout
 * shows by itself. Turning the spectrum off in Layar penuh does not turn it
 * off anywhere else.
 *
 * A toggle is a deliberate press, not a scroll, so it is written at once
 * rather than settled first the way a list position is.
 */

import { arrangement, layoutChoice } from '$lib/arrangement.svelte';
import { settings, updateDisplay } from '$lib/settings.svelte';
import { activeTheme } from '$lib/theme/index.svelte';
import { panelShown, withPanel, withPanels, type PanelId, type PanelPrefs } from '$lib/panels';

export { PANELS, needsAnalysis, shownPanels, type PanelId } from '$lib/panels';

/** Where the choices of the layout in force are kept. */
function kept(): { key: string | null; stored: PanelPrefs | undefined } {
	const choice = layoutChoice();
	if (choice === 'classic') return { key: null, stored: settings.value?.display.panels };
	const key = choice === 'signature' ? `signature:${activeTheme()?.signature?.id ?? ''}` : choice;
	return { key, stored: settings.value?.display.panelsBy?.[key] };
}

function write(key: string | null, next: PanelPrefs): void {
	if (key === null) {
		void updateDisplay({ panels: next });
		return;
	}
	const all = { ...(settings.value?.display.panelsBy ?? {}) };
	if (Object.keys(next).length === 0) delete all[key];
	else all[key] = next;
	void updateDisplay({ panelsBy: all });
}

/** Whether a panel is showing right now. */
export function shown(id: PanelId): boolean {
	return panelShown(kept().stored, id, arrangement().panels);
}

/** Shows or hides one panel, and remembers it for this layout. */
export function setPanel(id: PanelId, showing: boolean): void {
	const { key, stored } = kept();
	write(key, withPanel(stored, id, showing, arrangement().panels));
}

/** Turns one panel around. */
export const togglePanel = (id: PanelId): void => setPanel(id, !shown(id));

/** Decides several at once, for this layout. */
export function setPanels(patch: Partial<Record<PanelId, boolean>>): void {
	const { key, stored } = kept();
	write(key, withPanels(stored, patch, arrangement().panels));
}
