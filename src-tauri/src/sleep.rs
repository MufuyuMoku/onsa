//! The sleep timer (SPEC §3.5).
//!
//! Onsa can stop after a while, at the end of the track, or after a number
//! of tracks. Before it stops the volume comes down gently, then the volume
//! setting is put back, so the next play is not silent.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::player::Player;

/// The timer's state, sent whenever it changes.
pub const SLEEP_EVENT: &str = "player://sleep";

/// How often the timer looks at the clock and the player.
const TICK: Duration = Duration::from_millis(500);
/// Steps the fade is made of.
const FADE_STEP: Duration = Duration::from_millis(50);
/// Where the fade ends, in dB: quiet enough to be silence.
const FADE_FLOOR: f32 = -60.0;

/// When the timer should fire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum When {
    /// After this many minutes.
    Minutes {
        /// Minutes to wait.
        minutes: u32,
    },
    /// When the track that is playing ends.
    EndOfTrack,
    /// After this many more tracks.
    Tracks {
        /// Tracks to let play.
        tracks: u32,
    },
}

/// What to do once the timer fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    /// Pause where it is.
    #[default]
    Pause,
    /// Stop and forget the position.
    Stop,
    /// Close Onsa.
    Quit,
}

/// A timer as the user set it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    /// When to fire.
    pub when: When,
    /// What to do then.
    pub action: Action,
    /// How long the volume takes to come down, in seconds.
    pub fade_seconds: f32,
}

/// What the interface shows about the timer.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// Whether a timer is running.
    pub armed: bool,
    /// Seconds until it fires, when that can be known.
    pub seconds_left: Option<f64>,
    /// Tracks still to play, when counting tracks.
    pub tracks_left: Option<u32>,
    /// The timer as it was set.
    pub plan: Option<Plan>,
    /// Whether the volume is already coming down.
    pub fading: bool,
}

/// The sleep timer in the application state.
#[derive(Default)]
pub struct SleepTimer {
    status: Arc<Mutex<Status>>,
    cancel: Mutex<Option<Arc<AtomicBool>>>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl SleepTimer {
    /// Starts a timer, replacing one already running.
    pub fn arm(&self, app: &AppHandle, plan: Plan) {
        self.cancel();
        let cancel = Arc::new(AtomicBool::new(false));
        *lock(&self.cancel) = Some(cancel.clone());
        *lock(&self.status) = Status {
            armed: true,
            seconds_left: match plan.when {
                When::Minutes { minutes } => Some(f64::from(minutes) * 60.0),
                _ => None,
            },
            tracks_left: match plan.when {
                When::Tracks { tracks } => Some(tracks),
                _ => None,
            },
            plan: Some(plan),
            fading: false,
        };
        tracing::info!(?plan, "sleep timer set");
        self.publish(app);

        let status = self.status.clone();
        let handle = app.clone();
        let spawned = std::thread::Builder::new()
            .name("onsa-sleep".into())
            .spawn(move || run(handle, plan, status, cancel));
        if let Err(error) = spawned {
            tracing::error!("the sleep timer cannot start: {error}");
            self.cancel();
        }
    }

    /// Stops a running timer.
    pub fn cancel(&self) {
        if let Some(cancel) = lock(&self.cancel).take() {
            cancel.store(true, Ordering::Relaxed);
        }
        *lock(&self.status) = Status::default();
    }

    /// Stops a running timer and tells the interface.
    pub fn cancel_and_publish(&self, app: &AppHandle) {
        self.cancel();
        self.publish(app);
    }

    /// What the interface shows.
    pub fn status(&self) -> Status {
        *lock(&self.status)
    }

    fn publish(&self, app: &AppHandle) {
        let status = self.status();
        if let Err(error) = app.emit(SLEEP_EVENT, status) {
            tracing::debug!("sleep timer state not delivered: {error}");
        }
    }
}

/// Which queue entry is playing, if any.
fn current_track(app: &AppHandle) -> Option<usize> {
    app.try_state::<Player>()
        .and_then(|player| player.snapshot().current)
}

/// Watches the clock and the player until the timer is due.
fn run(app: AppHandle, plan: Plan, status: Arc<Mutex<Status>>, cancel: Arc<AtomicBool>) {
    let started = Instant::now();
    let fade = Duration::from_secs_f32(plan.fade_seconds.clamp(0.0, 60.0));
    let mut tracks_left = match plan.when {
        When::Tracks { tracks } => tracks,
        _ => 0,
    };
    let mut last_track = current_track(&app);

    loop {
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        std::thread::sleep(TICK);
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        let Some(player) = app.try_state::<Player>() else {
            return;
        };
        let snapshot = player.snapshot();

        // A track change counts down the "after N tracks" timer.
        let now_track = snapshot.current;
        if now_track != last_track {
            last_track = now_track;
            tracks_left = tracks_left.saturating_sub(1);
        }

        let left = match plan.when {
            When::Minutes { minutes } => {
                let total = Duration::from_secs(u64::from(minutes) * 60);
                total.checked_sub(started.elapsed()).unwrap_or_default()
            }
            When::EndOfTrack => remaining(&snapshot),
            When::Tracks { .. } => {
                if tracks_left > 1 {
                    Duration::MAX
                } else {
                    remaining(&snapshot)
                }
            }
        };

        {
            let mut status = lock(&status);
            status.seconds_left = (left != Duration::MAX).then_some(left.as_secs_f64());
            status.tracks_left = matches!(plan.when, When::Tracks { .. }).then_some(tracks_left);
        }
        let _ = app.emit(SLEEP_EVENT, *lock(&status));

        if left <= fade {
            break;
        }
    }

    lock(&status).fading = true;
    let _ = app.emit(SLEEP_EVENT, *lock(&status));
    fade_out(&app, fade, &cancel);
    if cancel.load(Ordering::Relaxed) {
        return;
    }
    finish(&app, plan.action);
    if let Some(timer) = app.try_state::<SleepTimer>() {
        timer.cancel_and_publish(&app);
    }
}

/// How much of the current track is left.
fn remaining(snapshot: &crate::player::Snapshot) -> Duration {
    match snapshot.duration {
        Some(duration) if snapshot.state != "stopped" => {
            Duration::from_secs_f64((duration - snapshot.position).max(0.0))
        }
        _ => Duration::MAX,
    }
}

/// Brings the volume down, then puts the setting back.
fn fade_out(app: &AppHandle, fade: Duration, cancel: &AtomicBool) {
    let Some(player) = app.try_state::<Player>() else {
        return;
    };
    let (_, _, dsp) = player.prefs();
    let from = dsp.volume_db.max(FADE_FLOOR);
    let steps = (fade.as_secs_f32() / FADE_STEP.as_secs_f32())
        .round()
        .max(1.0);
    for step in 1..=(steps as u32) {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let part = step as f32 / steps;
        let mut fading = dsp.clone();
        fading.volume_db = from + (FADE_FLOOR - from) * part;
        let _ = player.set_dsp(fading);
        std::thread::sleep(FADE_STEP);
    }
    // The listener's volume is a setting, not something a timer changes.
    let _ = player.set_dsp(dsp);
}

/// Carries out what the timer was set to do.
fn finish(app: &AppHandle, action: Action) {
    let Some(player) = app.try_state::<Player>() else {
        return;
    };
    tracing::info!(?action, "sleep timer fired");
    let result = match action {
        Action::Pause => player.pause(),
        Action::Stop | Action::Quit => player.stop(),
    };
    if let Err(code) = result {
        tracing::debug!("sleep timer command ignored: {code:?}");
    }
    if action == Action::Quit {
        app.exit(0);
    }
}
