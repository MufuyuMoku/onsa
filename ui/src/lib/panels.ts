/**
 * Which panels the Now Playing screen is showing (SPEC section 15, M14).
 *
 * The answer is data, not something each page works out for itself: a map
 * from a panel's name to whether it is showing. That is on purpose. The
 * layout presets of M16 arrange the same panels a different way, and a
 * preset is then a map of the same shape rather than another branch inside
 * a component.
 *
 * Only what somebody actually changed is stored. A panel nobody has touched
 * is not in the map at all, so a later change of heart about what a fresh
 * Onsa shows reaches everybody who never said otherwise.
 */

/** The panels, in the order they appear on the screen. */
export const PANELS = ['cover', 'details', 'lyrics', 'visualizer'] as const;

/** One of them. */
export type PanelId = (typeof PANELS)[number];

/** What a fresh Onsa shows: all of them. */
export const SHOWN_BY_DEFAULT: Record<PanelId, boolean> = {
	cover: true,
	details: true,
	lyrics: true,
	visualizer: true
};

/** What is stored: only the panels somebody has decided about. */
export type PanelPrefs = Record<string, boolean>;

/** Whether a panel is showing. */
export function panelShown(stored: PanelPrefs | undefined, id: PanelId): boolean {
	const said = stored?.[id];
	return typeof said === 'boolean' ? said : SHOWN_BY_DEFAULT[id];
}

/** The panels that are showing, in the order they appear. */
export function shownPanels(stored: PanelPrefs | undefined): PanelId[] {
	return PANELS.filter((id) => panelShown(stored, id));
}

/**
 * The stored map with these panels decided about.
 *
 * A panel put back the way it started leaves the map rather than sitting in
 * it as a copy of the default, so what is stored stays a record of choices.
 */
export function withPanels(stored: PanelPrefs | undefined, patch: Partial<Record<PanelId, boolean>>): PanelPrefs {
	const next: PanelPrefs = { ...(stored ?? {}) };
	for (const [id, shown] of Object.entries(patch) as [PanelId, boolean][]) {
		if (shown === SHOWN_BY_DEFAULT[id]) delete next[id];
		else next[id] = shown;
	}
	return next;
}

/** The same, for one panel. */
export const withPanel = (stored: PanelPrefs | undefined, id: PanelId, shown: boolean): PanelPrefs =>
	withPanels(stored, { [id]: shown });

/**
 * Whether the analysis tap is needed by anything on this screen.
 *
 * The visualizer is the only panel here that costs sound to draw, so
 * hiding it has to stop the tap and not merely hide the picture (SPEC
 * section 4.4, and the M5 rule M14 keeps).
 */
export const needsAnalysis = (stored: PanelPrefs | undefined): boolean =>
	panelShown(stored, 'visualizer');
