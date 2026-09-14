/**
 * Dragging a row to another place in the same list.
 *
 * Onsa cannot use HTML5 drag and drop: Tauri's own file-drop target owns the
 * native drag loop on Windows, and turning it off would cost the "drop a file
 * on the window to play it" of SPEC section 13. Tauri says as much in its
 * configuration — *disabling it is required to use HTML5 drag and drop on the
 * frontend on Windows* — so this does the same job with pointer events, which
 * nothing intercepts and which behave the same on both systems.
 *
 * The queue and the playlist share it; a list only says how many rows it has
 * and what to do when one lands somewhere else.
 */

/** How far a press has to travel before it counts as a drag, in pixels. */
const THRESHOLD = 4;
/** How close to the edge the pointer scrolls the list, in pixels. */
const EDGE = 32;
/** Most the list scrolls per frame while the pointer sits at the very edge. */
const SPEED = 14;

export interface ReorderOptions {
	/** How many rows the list holds right now. */
	count: () => number;
	/** Called once, when a row is let go somewhere else. */
	move: (from: number, to: number) => void;
	/** What one row looks like. Rows carry their place in `data-index`. */
	row?: string;
	/** Whether a row can be dragged at all (a smart playlist's cannot). */
	enabled?: () => boolean;
}

/**
 * Where a row ends up after being lifted out of `from` and put down at the
 * insertion point `place`.
 *
 * `place` counts the gaps between rows, so it runs from 0 to the row count;
 * dropping below the row it came from closes the gap the row left behind.
 * Returns `from` itself when the row would not actually move.
 */
export function settle(from: number, place: number, count: number): number {
	if (count <= 1) return from;
	const within = Math.max(0, Math.min(count, place));
	const to = within > from ? within - 1 : within;
	return Math.max(0, Math.min(count - 1, to));
}

/** The nearest thing above `node` that scrolls, if anything does. */
function scrollerOf(node: HTMLElement): HTMLElement | null {
	let at: HTMLElement | null = node;
	while (at) {
		const how = getComputedStyle(at).overflowY;
		if ((how === 'auto' || how === 'scroll') && at.scrollHeight > at.clientHeight) return at;
		at = at.parentElement;
	}
	return null;
}

/**
 * Makes every row inside `node` draggable to another place in the list.
 *
 * Rows are found by selector and read their own place from `data-index`, so
 * a list that only renders what is on screen works the same as one that
 * renders everything.
 */
export function reorderable(node: HTMLElement, options: ReorderOptions) {
	let current = options;
	let from = -1;
	let place = -1;
	let pointer = -1;
	let startY = 0;
	let dragging = false;
	let scroller: HTMLElement | null = null;
	let frame = 0;
	let towards = 0;

	const selector = () => current.row ?? '.row';
	const rows = () => [...node.querySelectorAll<HTMLElement>(selector())];
	const indexOf = (row: HTMLElement) => Number(row.dataset.index);

	/** The gap the pointer is hovering over, in row numbers. */
	function placeAt(y: number): number {
		const seen = rows();
		if (seen.length === 0) return from;
		let found = indexOf(seen[0]!);
		for (const row of seen) {
			const box = row.getBoundingClientRect();
			if (y > box.top + box.height / 2) found = indexOf(row) + 1;
		}
		return found;
	}

	/** Draws where the row would land: a line in the gap it is over. */
	function mark(): void {
		for (const row of rows()) {
			const index = indexOf(row);
			row.toggleAttribute('data-grabbed', dragging && index === from);
			if (!dragging || place < 0) {
				row.removeAttribute('data-drop');
			} else if (index === place) {
				row.setAttribute('data-drop', 'before');
			} else if (place >= current.count() && index === current.count() - 1) {
				row.setAttribute('data-drop', 'after');
			} else {
				row.removeAttribute('data-drop');
			}
		}
	}

	function clear(): void {
		dragging = false;
		from = -1;
		place = -1;
		towards = 0;
		node.removeAttribute('data-reordering');
		if (frame) cancelAnimationFrame(frame);
		frame = 0;
		mark();
	}

	/** Keeps the list moving while the pointer rests against its edge. */
	function creep(): void {
		frame = 0;
		if (!dragging || !scroller || towards === 0) return;
		scroller.scrollTop += towards;
		frame = requestAnimationFrame(creep);
	}

	function edgePull(y: number): void {
		towards = 0;
		if (!scroller) return;
		const box = scroller.getBoundingClientRect();
		if (y < box.top + EDGE) towards = -Math.ceil(((box.top + EDGE - y) / EDGE) * SPEED);
		else if (y > box.bottom - EDGE) towards = Math.ceil(((y - (box.bottom - EDGE)) / EDGE) * SPEED);
		if (towards !== 0 && frame === 0) frame = requestAnimationFrame(creep);
	}

	function onPointerDown(event: PointerEvent): void {
		if (event.button !== 0 || !event.isPrimary) return;
		if (current.enabled && !current.enabled()) return;
		const target = event.target as HTMLElement | null;
		// The controls inside a row belong to the row, not to the drag.
		if (!target || target.closest('button, a, input, select, textarea')) return;
		const row = target.closest<HTMLElement>(selector());
		if (!row || !node.contains(row)) return;
		const index = indexOf(row);
		if (!Number.isInteger(index)) return;
		from = index;
		place = -1;
		startY = event.clientY;
		pointer = event.pointerId;
		scroller = scrollerOf(row);
		// Deliberately not `setPointerCapture`: capturing would send the
		// click and the double-click to this element instead of the row,
		// and double-clicking a row is how it is played.
		listen(true);
	}

	function onPointerMove(event: PointerEvent): void {
		if (from < 0 || event.pointerId !== pointer) return;
		if (!dragging) {
			if (Math.abs(event.clientY - startY) < THRESHOLD) return;
			dragging = true;
			node.setAttribute('data-reordering', '');
		}
		// Once it is a drag, it is no longer a selection.
		event.preventDefault();
		place = placeAt(event.clientY);
		edgePull(event.clientY);
		mark();
	}

	function onPointerUp(event: PointerEvent): void {
		if (event.pointerId !== pointer) return;
		const was = { from, place, dragging };
		pointer = -1;
		listen(false);
		clear();
		if (!was.dragging || was.from < 0 || was.place < 0) return;
		const count = current.count();
		const to = settle(was.from, was.place, count);
		if (to !== was.from) current.move(was.from, to);
	}

	function listen(on: boolean): void {
		for (const [name, handler] of [
			['pointermove', onPointerMove],
			['pointerup', onPointerUp],
			['pointercancel', onPointerUp]
		] as const) {
			if (on) window.addEventListener(name, handler as EventListener);
			else window.removeEventListener(name, handler as EventListener);
		}
	}

	node.addEventListener('pointerdown', onPointerDown);

	return {
		update(next: ReorderOptions) {
			current = next;
		},
		destroy() {
			node.removeEventListener('pointerdown', onPointerDown);
			listen(false);
			if (frame) cancelAnimationFrame(frame);
		}
	};
}

/**
 * The keyboard's way of doing the same thing: Alt with an arrow moves the
 * row that has the focus, so reordering never needs a pointing device
 * (SPEC section 9.6). Returns whether the key was used.
 */
export function reorderKey(
	event: KeyboardEvent,
	index: number,
	count: number,
	move: (from: number, to: number) => void
): boolean {
	if (!event.altKey || event.ctrlKey || event.metaKey) return false;
	const step = event.key === 'ArrowUp' ? -1 : event.key === 'ArrowDown' ? 1 : 0;
	if (step === 0) return false;
	const to = index + step;
	if (to < 0 || to >= count) return true;
	event.preventDefault();
	move(index, to);
	return true;
}
