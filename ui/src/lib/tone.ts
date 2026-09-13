/**
 * Tone colour: how far a colour leans when the sound is heavy or bright
 * (SPEC section 9.5).
 *
 * The shift is a move between two colours the theme names, not a hue
 * rotation. Rotating a hue is easy to see on a saturated phosphor and
 * almost invisible on warm cream, and a theme knows which way its own
 * colours should lean; a move between two of its colours works for any
 * palette and can never land somewhere the theme did not choose.
 */

/** How far the listener wants the colour to move. */
export type ToneStrength = 'off' | 'subtle' | 'medium' | 'strong';

/** Weight of each setting. Medium is the whole span the theme allows. */
const WEIGHT: Record<ToneStrength, number> = {
	off: 0,
	subtle: 0.5,
	medium: 1,
	strong: 1.6
};

/** Sound around here keeps the theme's own colour. */
export const MID_HZ = 1200;
/** How far either side of the middle the span reaches, in octaves. */
export const OCTAVES = 2.2;

/** A colour as three channels, 0 to 255. */
type Rgb = [number, number, number];

/** Reads `#rgb`, `#rrggbb` or `rgb(...)`; anything else comes back null. */
export function parseColor(text: string): Rgb | null {
	const hex = text.trim().replace(/^#/, '');
	if (/^[0-9a-f]{3}$/i.test(hex)) {
		const [r, g, b] = [...hex].map((digit) => parseInt(digit + digit, 16));
		return [r ?? 0, g ?? 0, b ?? 0];
	}
	if (/^[0-9a-f]{6}$/i.test(hex)) {
		return [
			parseInt(hex.slice(0, 2), 16),
			parseInt(hex.slice(2, 4), 16),
			parseInt(hex.slice(4, 6), 16)
		];
	}
	const numbers = text.match(/-?\d+(\.\d+)?/g);
	if (/^rgba?\(/i.test(text.trim()) && numbers && numbers.length >= 3) {
		return [Number(numbers[0]), Number(numbers[1]), Number(numbers[2])];
	}
	return null;
}

/** Mixes two colours, `fraction` of the way from `from` to `to`. */
function mix(from: Rgb, to: Rgb, fraction: number): Rgb {
	const t = Math.max(0, Math.min(1, fraction));
	// Mixing through light rather than through the stored numbers keeps a
	// half-way mix looking half-way.
	const blend = (a: number, b: number) =>
		Math.round(Math.sqrt((1 - t) * (a / 255) ** 2 + t * (b / 255) ** 2) * 255);
	return [blend(from[0], to[0]), blend(from[1], to[1]), blend(from[2], to[2])];
}

/**
 * Where the sound sits: −1 for the heaviest, +1 for the brightest, 0 for a
 * sound around the middle. A centroid of zero means silence, which leans
 * nowhere.
 */
export function toneAmount(centroidHz: number): number {
	if (!(centroidHz > 0)) return 0;
	const octaves = Math.log2(centroidHz / MID_HZ) / OCTAVES;
	return Math.max(-1, Math.min(1, octaves));
}

/**
 * The colour `base` becomes at this point in the span. Heavy sound leans
 * towards `warm`, bright sound towards `cool`, and the strength says how
 * far along the way it gets.
 */
export function leaned(
	base: string,
	warm: string,
	cool: string,
	amount: number,
	strength: ToneStrength
): string {
	const from = parseColor(base);
	const to = parseColor(amount < 0 ? warm : cool);
	const weight = WEIGHT[strength] ?? 0;
	if (!from || !to || weight === 0 || amount === 0) return base;
	const [r, g, b] = mix(from, to, Math.abs(amount) * weight);
	return `rgb(${r}, ${g}, ${b})`;
}
