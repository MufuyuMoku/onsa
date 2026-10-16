/**
 * Asking the menu bar to open a menu from somewhere else in the window: the
 * button at the start of the task bar opens the first one (SPEC M16,
 * Jendela Biru).
 */

let asked = $state<{ index: number; at: number } | null>(null);

/** The last menu asked for, and when, so the same one can be asked twice. */
export function menuAsked(): { index: number; at: number } | null {
	return asked;
}

export function askForMenu(index: number): void {
	asked = { index, at: performance.now() };
}
