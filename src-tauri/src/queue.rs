//! The play queue as the application keeps it (SPEC §6.1).
//!
//! Entries are addressed by id, never by position. The interface acts on the
//! list it last drew, which may already have changed by the time the command
//! arrives; a position would then name a different track, while an id either
//! finds the entry the listener meant or finds nothing at all. The engine
//! plays from the same ids, so both sides mean the same thing by "the entry
//! that is playing".

use onsa_audio::{QueueId, QueueItem};
use onsa_library::TrackRow;

use crate::dto::QueuePlace;

/// One entry: a track, and what tells this entry apart from another entry of
/// the same track.
#[derive(Debug, Clone)]
pub struct Entry {
    /// Identity of the entry, for as long as it stays in the queue.
    pub id: QueueId,
    /// The track it plays.
    pub row: TrackRow,
}

impl Entry {
    fn new(row: TrackRow) -> Self {
        Self {
            id: QueueId::next(),
            row,
        }
    }
}

/// The queue, its shuffle order, and which entry is playing.
#[derive(Debug, Default)]
pub struct Queue {
    entries: Vec<Entry>,
    /// The order as listed, kept while shuffle is on so it can come back.
    listed: Vec<Entry>,
    shuffle: bool,
    current: Option<QueueId>,
}

impl Queue {
    /// The entries in playing order.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// How many entries there are.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Whether the queue is shuffled.
    pub fn shuffled(&self) -> bool {
        self.shuffle
    }

    /// The entry that is playing.
    pub fn current_id(&self) -> Option<QueueId> {
        self.current
    }

    /// Where the playing entry sits now.
    pub fn current_index(&self) -> Option<usize> {
        self.current.and_then(|id| self.index_of(id))
    }

    /// The track that is playing.
    pub fn current_row(&self) -> Option<&TrackRow> {
        self.current_index().map(|index| &self.entries[index].row)
    }

    /// Where an entry sits, if it is still in the queue.
    pub fn index_of(&self, id: QueueId) -> Option<usize> {
        self.entries.iter().position(|entry| entry.id == id)
    }

    /// The entry an id from the interface names, if the queue still holds
    /// it. An entry that has gone since the interface drew its list is
    /// simply not there, and the command does nothing.
    pub fn id_for(&self, entry: u64) -> Option<QueueId> {
        self.entries
            .iter()
            .map(|entry| entry.id)
            .find(|id| id.get() == entry)
    }

    /// The queue as the engine takes it. The library's values win over the
    /// file's tags: an edit kept in Onsa counts for album gapless and
    /// ReplayGain too.
    pub fn items(&self) -> Vec<QueueItem> {
        self.entries
            .iter()
            .map(|entry| QueueItem {
                id: entry.id,
                path: entry.row.path.clone().into(),
                album: entry.row.album.clone(),
                track_number: entry.row.track_number.map(u64::from),
            })
            .collect()
    }

    /// Replaces the queue and plays from `index`. With shuffle on, that
    /// track plays first and the rest follow in a random order; the order as
    /// listed is kept to go back to.
    pub fn set(&mut self, rows: Vec<TrackRow>, index: usize) {
        let entries: Vec<Entry> = rows.into_iter().map(Entry::new).collect();
        let index = index.min(entries.len().saturating_sub(1));
        self.current = entries.get(index).map(|entry| entry.id);
        if self.shuffle && entries.len() > 1 {
            self.listed = entries;
            let mut shuffled = self.listed.clone();
            let chosen = shuffled.remove(index);
            shuffle(&mut shuffled);
            shuffled.insert(0, chosen);
            self.entries = shuffled;
        } else {
            self.listed = Vec::new();
            self.entries = entries;
        }
    }

    /// Puts a stored queue back as it was listed (SPEC §13).
    pub fn restore(&mut self, rows: Vec<TrackRow>, index: usize, shuffle: bool) {
        self.shuffle = shuffle;
        self.listed = Vec::new();
        self.entries = rows.into_iter().map(Entry::new).collect();
        let index = index.min(self.entries.len().saturating_sub(1));
        self.current = self.entries.get(index).map(|entry| entry.id);
    }

    /// Adds tracks after the playing entry or at the end, without disturbing
    /// what is playing.
    pub fn insert(&mut self, rows: Vec<TrackRow>, place: QueuePlace) {
        let entries: Vec<Entry> = rows.into_iter().map(Entry::new).collect();
        if self.shuffle {
            self.listed.extend(entries.iter().cloned());
        }
        let at = match place {
            QueuePlace::Next => self.current_index().map_or(self.entries.len(), |at| at + 1),
            QueuePlace::End => self.entries.len(),
        };
        self.entries.splice(at..at, entries);
    }

    /// Removes one entry, wherever it sits.
    ///
    /// Removing the entry that is playing moves playback on to the entry
    /// that followed it: the listener took this track away, so hearing it
    /// out would be the wrong answer, and stopping altogether would be a
    /// surprise while the rest of the queue is still there. With nothing
    /// left after it, playback stops, unless the whole queue repeats, in
    /// which case it starts over. The engine follows the same rule.
    pub fn remove(&mut self, id: QueueId, wrap: bool) {
        let Some(index) = self.index_of(id) else {
            return;
        };
        self.entries.remove(index);
        if let Some(at) = self.listed.iter().position(|entry| entry.id == id) {
            self.listed.remove(at);
        }
        if self.current == Some(id) {
            self.current = self
                .entries
                .get(index)
                .or_else(|| wrap.then(|| self.entries.first()).flatten())
                .map(|entry| entry.id);
        }
    }

    /// Moves an entry to another place, as a drag in the queue panel does.
    /// What is playing does not change: it is the same entry wherever it
    /// lands.
    pub fn move_entry(&mut self, id: QueueId, to: usize) {
        let Some(from) = self.index_of(id) else {
            return;
        };
        let to = to.min(self.entries.len() - 1);
        if from == to {
            return;
        }
        let entry = self.entries.remove(from);
        self.entries.insert(to, entry);
    }

    /// Empties the queue.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.listed.clear();
        self.current = None;
    }

    /// Shuffles the queue, or puts the listed order back. The playing entry
    /// keeps playing either way, and moves to the front when shuffling.
    pub fn set_shuffle(&mut self, on: bool) {
        if self.shuffle == on {
            return;
        }
        self.shuffle = on;
        if self.entries.is_empty() {
            return;
        }
        if on {
            self.listed = self.entries.clone();
            match self.current_index() {
                Some(index) => {
                    let playing = self.entries.remove(index);
                    shuffle(&mut self.entries);
                    self.entries.insert(0, playing);
                }
                None => shuffle(&mut self.entries),
            }
        } else if !self.listed.is_empty() {
            self.entries = std::mem::take(&mut self.listed);
        }
    }

    /// Follows the engine: this is the entry it is playing.
    pub fn set_current(&mut self, id: QueueId) {
        if self.index_of(id).is_some() {
            self.current = Some(id);
        }
    }
}

/// Fisher-Yates with a small generator seeded from the clock. Shuffling a
/// queue needs no cryptographic randomness, and this keeps the application
/// free of another dependency.
fn shuffle<T>(entries: &mut [T]) {
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0x9E37_79B9_7F4A_7C15, |since| since.as_nanos() as u64 | 1);
    let mut next = move || {
        seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = seed;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    };
    for index in (1..entries.len()).rev() {
        entries.swap(index, (next() % (index as u64 + 1)) as usize);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(count: i64) -> Vec<TrackRow> {
        (0..count)
            .map(|id| TrackRow {
                id,
                path: format!("/music/{id}.flac"),
                title: Some(format!("track {id}")),
                ..TrackRow::default()
            })
            .collect()
    }

    /// The tracks in queue order, by library id.
    fn order(queue: &Queue) -> Vec<i64> {
        queue.entries().iter().map(|entry| entry.row.id).collect()
    }

    fn playing(queue: &Queue) -> Option<i64> {
        queue.current_row().map(|row| row.id)
    }

    #[test]
    fn removing_the_playing_entry_moves_on_to_the_next() {
        let mut queue = Queue::default();
        queue.set(rows(3), 1);
        assert_eq!(playing(&queue), Some(1));

        let id = queue.current_id().expect("playing");
        queue.remove(id, false);
        assert_eq!(order(&queue), vec![0, 2]);
        assert_eq!(playing(&queue), Some(2), "the next track should take over");
        assert_eq!(queue.current_index(), Some(1));
    }

    #[test]
    fn removing_the_playing_entry_at_the_end_stops() {
        let mut queue = Queue::default();
        queue.set(rows(3), 2);
        let id = queue.current_id().expect("playing");
        queue.remove(id, false);
        assert_eq!(playing(&queue), None, "nothing follows the last entry");

        // Unless the whole queue repeats, which starts it over.
        let mut queue = Queue::default();
        queue.set(rows(3), 2);
        let id = queue.current_id().expect("playing");
        queue.remove(id, true);
        assert_eq!(playing(&queue), Some(0));
    }

    #[test]
    fn removing_everything_after_the_playing_entry_leaves_it_playing() {
        let mut queue = Queue::default();
        queue.set(rows(5), 1);
        let ids: Vec<QueueId> = queue.entries().iter().map(|entry| entry.id).collect();
        for index in [4, 3, 2] {
            queue.remove(ids[index], false);
        }
        assert_eq!(order(&queue), vec![0, 1]);
        assert_eq!(playing(&queue), Some(1));
    }

    #[test]
    fn quick_removals_never_take_the_wrong_entry() {
        // The interface sends the ids it drew, all from the same list, as
        // fast as the listener can click. Each one removes exactly the
        // entry the listener clicked, whatever else has gone meanwhile.
        let mut queue = Queue::default();
        queue.set(rows(6), 3);
        let ids: Vec<QueueId> = queue.entries().iter().map(|entry| entry.id).collect();

        for id in [ids[0], ids[5], ids[1], ids[4]] {
            queue.remove(id, false);
        }
        assert_eq!(order(&queue), vec![2, 3]);
        assert_eq!(playing(&queue), Some(3), "the playing track was lost");

        // Even an entry removed twice, as a double click would.
        queue.remove(ids[0], false);
        assert_eq!(order(&queue), vec![2, 3]);
    }

    #[test]
    fn a_command_from_a_list_already_out_of_date_still_names_the_right_track() {
        // This is the click that went wrong before: the interface draws a
        // list, the listener clicks its fourth row, and by the time the
        // command arrives two entries have gone. A position would name a
        // different track; the id names the one they clicked.
        let mut queue = Queue::default();
        queue.set(rows(6), 0);
        let clicked = queue.entries()[3].id;
        let clicked_track = queue.entries()[3].row.id;

        let first = queue.entries()[0].id;
        let second = queue.entries()[1].id;
        queue.remove(first, false);
        queue.remove(second, false);
        assert_ne!(
            queue.entries()[3].row.id,
            clicked_track,
            "the fourth row is another track now"
        );

        queue.remove(clicked, false);
        assert!(!order(&queue).contains(&clicked_track));
        assert_eq!(order(&queue), vec![2, 4, 5]);
    }

    #[test]
    fn emptying_the_queue_while_playing_leaves_nothing_playing() {
        let mut queue = Queue::default();
        queue.set(rows(4), 2);
        queue.clear();
        assert!(queue.is_empty());
        assert_eq!(playing(&queue), None);
    }

    #[test]
    fn a_drag_keeps_the_same_entry_playing() {
        let mut queue = Queue::default();
        queue.set(rows(4), 1);
        let ids: Vec<QueueId> = queue.entries().iter().map(|entry| entry.id).collect();

        // The playing entry itself is dragged to the end.
        queue.move_entry(ids[1], 3);
        assert_eq!(order(&queue), vec![0, 2, 3, 1]);
        assert_eq!(playing(&queue), Some(1));
        assert_eq!(queue.current_index(), Some(3));

        // Another entry is dragged across it.
        queue.move_entry(ids[0], 3);
        assert_eq!(order(&queue), vec![2, 3, 1, 0]);
        assert_eq!(playing(&queue), Some(1));
        assert_eq!(queue.current_index(), Some(2));
    }

    #[test]
    fn play_next_and_add_to_the_end_go_where_they_say() {
        let mut queue = Queue::default();
        queue.set(rows(3), 1);
        let mut more = rows(5);
        queue.insert(vec![more.remove(4)], QueuePlace::Next);
        assert_eq!(order(&queue), vec![0, 1, 4, 2]);
        queue.insert(vec![more.remove(3)], QueuePlace::End);
        assert_eq!(order(&queue), vec![0, 1, 4, 2, 3]);
        assert_eq!(playing(&queue), Some(1), "adding must not move playback");
    }

    #[test]
    fn shuffle_keeps_the_playing_entry_and_the_listed_order() {
        let mut queue = Queue::default();
        queue.set(rows(8), 5);
        queue.set_shuffle(true);
        assert_eq!(playing(&queue), Some(5));
        assert_eq!(queue.current_index(), Some(0), "it leads the shuffle");
        assert_eq!(queue.len(), 8);

        // An entry removed while shuffled stays removed when the listed
        // order comes back.
        let gone = queue.entries()[3].id;
        let gone_row = queue.entries()[3].row.id;
        queue.remove(gone, false);
        queue.set_shuffle(false);
        assert_eq!(playing(&queue), Some(5));
        assert_eq!(queue.len(), 7);
        assert!(!order(&queue).contains(&gone_row));
        assert_eq!(
            order(&queue),
            (0..8).filter(|id| *id != gone_row).collect::<Vec<_>>()
        );
    }

    #[test]
    fn the_engine_has_the_last_word_on_what_is_playing() {
        let mut queue = Queue::default();
        queue.set(rows(3), 0);
        let last = queue.entries()[2].id;
        queue.set_current(last);
        assert_eq!(playing(&queue), Some(2));

        // An entry that has left the queue cannot become the current one.
        queue.remove(last, false);
        assert_eq!(playing(&queue), None);
        queue.set_current(last);
        assert_eq!(playing(&queue), None);
    }
}
