/**
 * How the window is laid out at the width it happens to be (SPEC section 9.2).
 *
 * Onsa is an instrument panel, and a panel that is too narrow drops
 * instruments rather than letting them overlap. Everything that changes
 * shape does so at a width named here, so the steps are the same wherever
 * they are read, and a component that adapts to its own box measures that
 * box rather than guessing from the window.
 */

/** Where the shell changes shape, in CSS pixels. */
export const STEPS = {
	/** At or above this, the queue is a column of its own. */
	queue: 1160,
	/** At or above this, the sidebar shows its words. */
	sidebar: 1040,
	/** At or above this, the sidebar is a strip of icons; below, it hides. */
	sidebarIcons: 880,
	/** At or above this, the help panel stands beside the page. */
	help: 900
} as const;

/** How the sidebar is showing. */
export type SidebarMode = 'full' | 'icons' | 'hidden';

/** What the right-hand panel is showing (SPEC section 9.2). */
export type PanelTab = 'queue' | 'lyrics';

let width = $state(1180);
let queueOpen = $state(false);
let menuOpen = $state(false);
let helpOpen = $state(false);
let helpTopic = $state<string | null>(null);
let panelTab = $state<PanelTab>('queue');

export const layout = {
	/** Width of the window, in CSS pixels. */
	get width() {
		return width;
	},
	/** Whether the queue has a column of its own. */
	get queueDocked() {
		return width >= STEPS.queue;
	},
	get sidebar(): SidebarMode {
		if (width >= STEPS.sidebar) return 'full';
		return width >= STEPS.sidebarIcons ? 'icons' : 'hidden';
	},
	/** Whether the queue is on screen: docked, or opened over the content. */
	get queueShown() {
		return this.queueDocked || queueOpen;
	},
	/** Whether the queue is covering the content rather than beside it. */
	get queueFloating() {
		return !this.queueDocked && queueOpen;
	},
	/** Whether the sidebar is covering the content. */
	get menuFloating() {
		return this.sidebar === 'hidden' && menuOpen;
	},
	/** Which of the panel's two tabs is showing. */
	get panelTab(): PanelTab {
		return panelTab;
	},
	/** Whether the help for this page is open. */
	get helpOpen() {
		return helpOpen;
	},
	/**
	 * A help entry asked for by name rather than by where the window is —
	 * the contents on the help page can point at the window's own furniture,
	 * which is nowhere to navigate to.
	 */
	get helpTopic() {
		return helpTopic;
	},
	/**
	 * Whether the help has room of its own beside the page. Below that
	 * width it lies over the page instead — still without a scrim, because
	 * help that stops the page being used is not help.
	 */
	get helpDocked() {
		return width >= STEPS.help;
	}
};

/** Shows one of the panel's tabs, opening the panel if it is away. */
export function showInPanel(tab: PanelTab): void {
	panelTab = tab;
	if (!layout.queueDocked) queueOpen = true;
}

/** Opens or closes the help for the page being shown. */
export function toggleHelp(): void {
	helpOpen = !helpOpen;
}

/** Opens the help, for somebody arriving from the help page's contents. */
export function openHelp(id?: string): void {
	helpTopic = id ?? null;
	helpOpen = true;
}

/** Back to the help of wherever the window is. Called when it moves. */
export function forgetHelpTopic(): void {
	helpTopic = null;
}

/** Puts the help away. */
export function closeHelp(): void {
	helpOpen = false;
	helpTopic = null;
}

/** Opens or closes the queue panel. */
export function toggleQueue(): void {
	queueOpen = !queueOpen;
}

/** Opens or closes the navigation drawer. */
export function toggleMenu(): void {
	menuOpen = !menuOpen;
}

/** Puts away whatever is covering the content. */
export function closeOverlays(): void {
	queueOpen = false;
	menuOpen = false;
}

/** Follows the window's width. Call once, from the root. */
export function followWidth(): void {
	$effect(() => {
		const read = () => {
			width = window.innerWidth;
			// A panel that covers the content has no business staying open
			// once there is room for it beside the content again.
			if (width >= STEPS.queue) queueOpen = false;
			if (width >= STEPS.sidebarIcons) menuOpen = false;
		};
		read();
		window.addEventListener('resize', read);
		return () => window.removeEventListener('resize', read);
	});
}

/**
 * Follows an element's own width. For panels whose room depends on what else
 * is on screen, not only on the window.
 *
 * ```svelte
 * <div use:measure={(w) => (width = w)}>
 * ```
 */
export function measure(node: HTMLElement, report: (width: number) => void) {
	report(node.getBoundingClientRect().width);
	const observer = new ResizeObserver((entries) => {
		const box = entries[0]?.contentRect;
		if (box) report(box.width);
	});
	observer.observe(node);
	return {
		destroy() {
			observer.disconnect();
		}
	};
}
