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
	sidebarIcons: 880
} as const;

/** How the sidebar is showing. */
export type SidebarMode = 'full' | 'icons' | 'hidden';

let width = $state(1180);
let queueOpen = $state(false);
let menuOpen = $state(false);

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
	}
};

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
