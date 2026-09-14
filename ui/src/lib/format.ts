/**
 * Numbers as the panel shows them. Units are technical symbols (dB, Hz,
 * kHz), the same in every language; words come from the dictionaries.
 */

const MINUS = '−';

/** Seconds as `m:ss`, or `h:mm:ss` from an hour on. */
export function clock(seconds: number | null | undefined): string {
	if (seconds === null || seconds === undefined || !Number.isFinite(seconds)) return '--:--';
	const total = Math.max(0, Math.floor(seconds));
	const hours = Math.floor(total / 3600);
	const minutes = Math.floor((total % 3600) / 60);
	const rest = String(total % 60).padStart(2, '0');
	return hours > 0 ? `${hours}:${String(minutes).padStart(2, '0')}:${rest}` : `${minutes}:${rest}`;
}

/** A level or gain in dB with its sign, for example `−6.2 dB`. */
export function db(value: number, digits = 1): string {
	if (!Number.isFinite(value)) return `${MINUS}∞ dB`;
	const rounded = Number(value.toFixed(digits));
	const sign = rounded > 0 ? '+' : rounded < 0 ? MINUS : '';
	return `${sign}${Math.abs(rounded).toFixed(digits)} dB`;
}

/** A sample rate, for example `44.1 kHz` or `48 kHz`. */
export function rate(hz: number): string {
	const khz = hz / 1000;
	return `${Number.isInteger(khz) ? khz : khz.toFixed(1)} kHz`;
}

/** A frequency, for example `62 Hz` or `1.5 kHz`. */
export function frequency(hz: number): string {
	if (hz >= 1000) {
		const khz = hz / 1000;
		return `${Number.isInteger(khz) ? khz : khz.toFixed(1)} kHz`;
	}
	return `${Math.round(hz)} Hz`;
}

/** Milliseconds as `12 ms`. */
export function millis(value: number): string {
	return `${Math.round(value)} ms`;
}

/** Seconds as `2.5 s`. */
export function secondsLabel(value: number): string {
	return `${Number.isInteger(value) ? value : value.toFixed(1)} s`;
}

/**
 * The last part of a path, whichever way its separators lean.
 *
 * It lives here rather than in each component that wants it: seven copies
 * of this line is how one of them ended up splitting on forward slashes
 * alone, which leaves a whole Windows path where a file name belongs.
 */
export function fileName(path: string): string {
	return path.split(/[\\/]/).pop() ?? path;
}
