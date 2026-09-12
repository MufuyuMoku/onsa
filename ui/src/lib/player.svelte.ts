/**
 * What the interface knows about playback, kept up to date by the backend's
 * player events (SPEC section 2).
 */

import {
	clearQueue,
	enqueue,
	EVENTS,
	moveInQueue,
	on,
	play,
	playerQueue,
	playerSnapshot,
	removeFromQueue,
	setShuffle,
	sleepArm,
	sleepCancel,
	sleepStatus,
	type Meter,
	type PlayContext,
	type PlayerSnapshot,
	type Position,
	type Queue,
	type QueuePlace,
	type RepeatKind,
	type SleepPlan,
	type SleepStatus
} from '$lib/backend';
import { settings, updatePlayback } from '$lib/settings.svelte';

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
let sleep = $state<SleepStatus>({
	armed: false,
	secondsLeft: null,
	tracksLeft: null,
	plan: null,
	fading: false
});

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
	},
	/** Where the sleep timer stands. */
	get sleep() {
		return sleep;
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
	sleep = await sleepStatus();
	await on<null>(EVENTS.playerQueue, () => {
		void refreshQueue();
	});
	await on<SleepStatus>(EVENTS.sleep, (next) => {
		sleep = next;
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
	await refreshQueue();
}

/** Reads the queue again. */
export async function refreshQueue(): Promise<void> {
	queue = await playerQueue();
}

/** Adds tracks after the playing one, or at the end (SPEC section 6.1). */
export async function addToQueue(context: PlayContext, place: QueuePlace): Promise<void> {
	await enqueue(context, place);
	await refreshQueue();
}

/**
 * Removes one queue entry, named by its id. The list the listener clicked
 * may already be out of date, and an id still means the entry they clicked
 * (SPEC section 6.1).
 */
export async function removeQueueEntry(entry: number): Promise<void> {
	await removeFromQueue(entry);
	await refreshQueue();
}

/** Moves a queue entry, as a drag does. */
export async function moveQueueEntry(entry: number, to: number): Promise<void> {
	await moveInQueue(entry, to);
	await refreshQueue();
}

/** Empties the queue. */
export async function emptyQueue(): Promise<void> {
	await clearQueue();
	await refreshQueue();
}

/** Shuffles the queue, or puts the listed order back. */
export async function toggleShuffle(): Promise<void> {
	await setShuffle(!(snapshot?.shuffle ?? false));
	await refreshQueue();
}

/** Steps through off, the whole queue, and one track. */
export async function cycleRepeat(): Promise<void> {
	const order: RepeatKind[] = ['off', 'all', 'one'];
	const now = settings.value?.playback.repeat ?? 'off';
	const next = order[(order.indexOf(now) + 1) % order.length] ?? 'off';
	await updatePlayback({ repeat: next });
}

/** Sets the sleep timer (SPEC section 3.5). */
export async function armSleep(plan: SleepPlan): Promise<void> {
	await sleepArm(plan);
}

/** Cancels the sleep timer. */
export async function cancelSleep(): Promise<void> {
	await sleepCancel();
}
