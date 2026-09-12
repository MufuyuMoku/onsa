/**
 * What the interface knows about playback, kept up to date by the backend's
 * player events (SPEC section 2).
 */

import {
	EVENTS,
	on,
	play,
	playerQueue,
	playerSnapshot,
	type Meter,
	type PlayContext,
	type PlayerSnapshot,
	type Position,
	type Queue
} from '$lib/backend';

/** Meters at rest. */
const FLOOR: Meter = { peakDb: [-120, -120], clip: false, limiting: false };
/** How long CLIP and LIM stay lit after the event, so a short one is seen. */
const HOLD_MS = 1200;

let snapshot = $state<PlayerSnapshot | null>(null);
let position = $state(0);
let meter = $state<Meter>(FLOOR);
let clipLit = $state(false);
let limitLit = $state(false);
let queue = $state<Queue>({ items: [], current: null });

let clipTimer: ReturnType<typeof setTimeout> | undefined;
let limitTimer: ReturnType<typeof setTimeout> | undefined;

export const player = {
	get snapshot() {
		return snapshot;
	},
	/** Seconds into the current track. */
	get position() {
		return position;
	},
	get meter() {
		return meter;
	},
	/** Clipping was seen within the hold time. */
	get clip() {
		return clipLit;
	},
	/** The limiter worked within the hold time. */
	get limiting() {
		return limitLit;
	},
	get queue() {
		return queue;
	}
};

/** Reads the player and follows its events. */
export async function initPlayer(): Promise<void> {
	snapshot = await playerSnapshot();
	position = snapshot.position;
	queue = await playerQueue();
	await on<PlayerSnapshot>(EVENTS.playerState, (next) => {
		snapshot = next;
		position = next.position;
		// Analysis stops while nothing plays, so the meters must not freeze.
		if (next.state !== 'playing') meter = FLOOR;
	});
	await on<Position>(EVENTS.playerPosition, (next) => {
		position = next.seconds;
	});
	await on<Meter>(EVENTS.playerMeter, (next) => {
		meter = next;
		if (next.clip) {
			clipLit = true;
			clearTimeout(clipTimer);
			clipTimer = setTimeout(() => (clipLit = false), HOLD_MS);
		}
		if (next.limiting) {
			limitLit = true;
			clearTimeout(limitTimer);
			limitTimer = setTimeout(() => (limitLit = false), HOLD_MS);
		}
	});
}

/** Replaces the queue and plays. */
export async function playContext(context: PlayContext): Promise<void> {
	await play(context);
	queue = await playerQueue();
}
