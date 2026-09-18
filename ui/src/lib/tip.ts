/**
 * A tooltip Onsa draws itself, so the theme can decide what one looks like
 * (SPEC §9.3).
 *
 * The system's own `title` tooltip is fine for a long path — it wraps, it
 * waits, and it is what people expect from a file name. This is for the
 * short notes on the instrument itself, where a box drawn in the theme's
 * material belongs with everything around it.
 *
 * It is an action rather than a component so that a control keeps its own
 * markup: `use:tip={'...'}` adds nothing to the element but listeners.
 *
 * Nothing is drawn until somebody hovers or focuses the element, and it is
 * taken away again on leave, blur, Escape, or the element leaving the page.
 * A pointer that is only passing through does not summon one: it waits.
 */

/** How long a pointer has to rest before a tip appears. */
const WAIT = 400;

/** How far from the element the tip sits. */
const GAP = 8;

let shown: HTMLElement | null = null;
let timer: ReturnType<typeof setTimeout> | null = null;

/** Takes away whatever tip is showing. */
export function hideTip(): void {
	if (timer !== null) {
		clearTimeout(timer);
		timer = null;
	}
	shown?.remove();
	shown = null;
}

/** Draws a tip for `anchor`, keeping it inside the window. */
function showTip(anchor: Element, text: string): void {
	hideTip();
	const tip = document.createElement('div');
	tip.className = 'tip';
	tip.setAttribute('role', 'tooltip');
	tip.textContent = text;
	document.body.appendChild(tip);

	const box = anchor.getBoundingClientRect();
	const own = tip.getBoundingClientRect();
	// Under the element by default, above it when there is no room below.
	const below = box.bottom + GAP;
	const top = below + own.height > window.innerHeight ? box.top - own.height - GAP : below;
	const left = Math.min(
		Math.max(4, box.left),
		Math.max(4, window.innerWidth - own.width - 4)
	);
	tip.style.top = `${Math.max(4, top)}px`;
	tip.style.left = `${left}px`;
	shown = tip;
}

/**
 * Gives an element a tooltip in the theme's own material.
 *
 * ```svelte
 * <span use:tip={t('match.sureTitle')}>96%</span>
 * ```
 */
export function tip(node: HTMLElement, text: string) {
	let current = text;

	const open = () => {
		if (!current) return;
		if (timer !== null) clearTimeout(timer);
		timer = setTimeout(() => showTip(node, current), WAIT);
	};
	const close = () => hideTip();
	const onKey = (event: KeyboardEvent) => {
		if (event.key === 'Escape') hideTip();
	};

	node.addEventListener('pointerenter', open);
	node.addEventListener('pointerleave', close);
	node.addEventListener('pointerdown', close);
	node.addEventListener('focusin', open);
	node.addEventListener('focusout', close);
	node.addEventListener('keydown', onKey);

	return {
		update(next: string) {
			current = next;
			if (shown) showTip(node, current);
		},
		destroy() {
			close();
			node.removeEventListener('pointerenter', open);
			node.removeEventListener('pointerleave', close);
			node.removeEventListener('pointerdown', close);
			node.removeEventListener('focusin', open);
			node.removeEventListener('focusout', close);
			node.removeEventListener('keydown', onKey);
		}
	};
}
