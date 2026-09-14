/**
 * Carrying tracks from one place in the interface to another: a row of the
 * track list onto a playlist in the sidebar.
 *
 * The same reason as reordering a list — Tauri's file-drop target owns the
 * native drag loop on Windows — means this cannot be HTML5 drag and drop
 * either, so it is built on pointer events as well, and the two share the
 * same feel: a press has to travel before it counts as a drag.
 */

import type { PlayContext } from '$lib/backend';

/** How far a press has to travel before it counts as a drag, in pixels. */
const THRESHOLD = 5;

/** What is being carried, and what it is called while it is in the air. */
export interface Carried {
	context: PlayContext;
	label: string;
	count: number;
}

let carried = $state<Carried | null>(null);
let at = $state<{ x: number; y: number } | null>(null);
/** The drop target under the pointer, named by its own key. */
let over = $state<string | null>(null);

export const carry = {
	/** What is in the air, if anything. */
	get what() {
		return carried;
	},
	/** Where the pointer is, so the badge can follow it. */
	get at() {
		return at;
	},
	/** Which drop target the pointer is over. */
	get over() {
		return over;
	}
};

/** Every place something can be dropped, by the key it answers to. */
const targets = new Map<string, (what: Carried) => void>();

/**
 * Marks an element as somewhere tracks can be dropped. The key is what the
 * carrier reports as being hovered, so the element can light up.
 */
export function dropTarget(
	node: HTMLElement,
	options: { key: string; accept: (what: Carried) => void }
) {
	let key = options.key;
	node.dataset.dropTarget = key;
	targets.set(key, options.accept);
	return {
		update(next: { key: string; accept: (what: Carried) => void }) {
			if (next.key !== key) {
				targets.delete(key);
				key = next.key;
				node.dataset.dropTarget = key;
			}
			targets.set(key, next.accept);
		},
		destroy() {
			targets.delete(key);
		}
	};
}

/** The drop target under a point on the screen, if there is one. */
function targetAt(x: number, y: number): string | null {
	for (const node of document.elementsFromPoint(x, y)) {
		if (node instanceof HTMLElement && node.dataset.dropTarget) return node.dataset.dropTarget;
	}
	return null;
}

/**
 * Starts carrying something, from the press that began it. Nothing happens
 * until the pointer has travelled far enough for a press to be a drag, so a
 * click and a double-click still reach the row underneath.
 */
export function startCarry(event: PointerEvent, what: () => Carried | null): void {
	if (event.button !== 0 || !event.isPrimary) return;
	const startX = event.clientX;
	const startY = event.clientY;
	const pointer = event.pointerId;
	let lifted = false;

	function move(next: PointerEvent): void {
		if (next.pointerId !== pointer) return;
		if (!lifted) {
			const far =
				Math.abs(next.clientX - startX) > THRESHOLD || Math.abs(next.clientY - startY) > THRESHOLD;
			if (!far) return;
			const load = what();
			if (!load) return stop();
			lifted = true;
			carried = load;
		}
		next.preventDefault();
		at = { x: next.clientX, y: next.clientY };
		over = targetAt(next.clientX, next.clientY);
	}

	function up(next: PointerEvent): void {
		if (next.pointerId !== pointer) return;
		const load = carried;
		const where = over;
		stop();
		if (!load || !where) return;
		targets.get(where)?.(load);
	}

	function stop(): void {
		window.removeEventListener('pointermove', move);
		window.removeEventListener('pointerup', up);
		window.removeEventListener('pointercancel', stop);
		carried = null;
		at = null;
		over = null;
	}

	window.addEventListener('pointermove', move);
	window.addEventListener('pointerup', up);
	window.addEventListener('pointercancel', stop);
}
