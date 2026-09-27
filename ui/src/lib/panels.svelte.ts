/**
 * The panel state as the interface holds it (SPEC section 15, M14).
 *
 * The rule itself lives in `panels.ts`, which knows nothing about state or
 * storage and can be read and tested on its own. This is only the thin
 * layer that reads it out of the display settings and writes a change back.
 *
 * A toggle is a deliberate press, not a scroll, so it is written at once
 * rather than settled first the way a list position is.
 */

import { settings, updateDisplay } from '$lib/settings.svelte';
import { panelShown, withPanel, withPanels, type PanelId } from '$lib/panels';

export { PANELS, needsAnalysis, shownPanels, type PanelId } from '$lib/panels';

/** Whether a panel is showing right now. */
export function shown(id: PanelId): boolean {
	return panelShown(settings.value?.display.panels, id);
}

/** Shows or hides one panel, and remembers it. */
export function setPanel(id: PanelId, showing: boolean): void {
	void updateDisplay({ panels: withPanel(settings.value?.display.panels, id, showing) });
}

/** Turns one panel around. */
export const togglePanel = (id: PanelId): void => setPanel(id, !shown(id));

/**
 * Decides several at once.
 *
 * Nothing calls this yet: it is the door the layout presets of M16 come
 * through, and it is here so that a preset arranges the same data rather
 * than growing a second way of saying what is on screen.
 */
export function setPanels(patch: Partial<Record<PanelId, boolean>>): void {
	void updateDisplay({ panels: withPanels(settings.value?.display.panels, patch) });
}
