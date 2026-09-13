//! The library's database schema, as numbered migrations (SPEC §5.1).
//!
//! Every schema change is a new migration appended to [`MIGRATIONS`]; an
//! applied migration is never edited. The number of applied migrations is
//! kept in SQLite's `user_version`.
//!
//! Version 1 holds what the library itself needs (M3), version 2 the
//! playlists (M6). Lyrics, the scrobble queue and downloads arrive as later
//! migrations with the milestones that use them.

/// The migrations, in order. Index 0 is version 1.
pub const MIGRATIONS: &[&str] = &[V1, V2];

const V1: &str = r#"
CREATE TABLE folders (
    id          INTEGER PRIMARY KEY,
    path        TEXT NOT NULL UNIQUE,
    added_at    INTEGER NOT NULL
);

CREATE TABLE covers (
    id          INTEGER PRIMARY KEY,
    hash        TEXT NOT NULL UNIQUE,     -- SHA-256 of the image bytes, hex
    width       INTEGER NOT NULL,
    height      INTEGER NOT NULL,
    thumb_128   TEXT NOT NULL,            -- file name in the cover cache
    thumb_512   TEXT NOT NULL
);

CREATE TABLE albums (
    id           INTEGER PRIMARY KEY,
    title        TEXT NOT NULL,
    album_artist TEXT NOT NULL,
    year         INTEGER,
    cover_id     INTEGER REFERENCES covers(id) ON DELETE SET NULL,
    UNIQUE (title, album_artist)
);

CREATE TABLE tracks (
    id              INTEGER PRIMARY KEY,
    path            TEXT NOT NULL UNIQUE,
    folder_id       INTEGER NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
    mtime           INTEGER NOT NULL,     -- milliseconds since the Unix epoch
    size            INTEGER NOT NULL,
    duration_ms     INTEGER,
    codec           TEXT,
    sample_rate     INTEGER,
    bit_depth       INTEGER,
    channels        INTEGER,
    bitrate         INTEGER,              -- kbit/s
    title           TEXT,
    artist          TEXT,
    album           TEXT,
    album_artist    TEXT,
    track_number    INTEGER,
    track_total     INTEGER,
    disc_number     INTEGER,
    disc_total      INTEGER,
    year            INTEGER,
    genre           TEXT,
    composer        TEXT,
    rg_track_gain   REAL,
    rg_track_peak   REAL,
    rg_album_gain   REAL,
    rg_album_peak   REAL,
    album_id        INTEGER REFERENCES albums(id) ON DELETE SET NULL,
    cover_id        INTEGER REFERENCES covers(id) ON DELETE SET NULL,
    status          TEXT NOT NULL DEFAULT 'ok'
                    CHECK (status IN ('ok', 'missing', 'failed')),
    added_at        INTEGER NOT NULL
);
CREATE INDEX tracks_folder ON tracks(folder_id);
CREATE INDEX tracks_album  ON tracks(album_id);
CREATE INDEX tracks_status ON tracks(status);

-- Edits kept in Onsa (SPEC §8): one row per track and field.
CREATE TABLE overrides (
    track_id    INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    field       TEXT NOT NULL,
    value       TEXT,
    unwritten   INTEGER NOT NULL DEFAULT 1,   -- not yet written to the file
    PRIMARY KEY (track_id, field)
);

CREATE TABLE stats (
    track_id     INTEGER PRIMARY KEY REFERENCES tracks(id) ON DELETE CASCADE,
    play_count   INTEGER NOT NULL DEFAULT 0,
    skip_count   INTEGER NOT NULL DEFAULT 0,
    last_played  INTEGER,
    rating       INTEGER NOT NULL DEFAULT 0 CHECK (rating BETWEEN 0 AND 5)
);

CREATE TABLE plays (
    id           INTEGER PRIMARY KEY,
    track_id     INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    started_at   INTEGER NOT NULL,
    listened_ms  INTEGER NOT NULL
);
CREATE INDEX plays_track ON plays(track_id);

CREATE TABLE settings (
    key    TEXT PRIMARY KEY,
    value  TEXT NOT NULL                      -- JSON
);

-- The values shown to the user: an override when there is one, otherwise
-- what the file says (SPEC §5.1).
CREATE VIEW track_view AS
SELECT t.id, t.path, t.folder_id, t.duration_ms, t.codec, t.sample_rate,
       t.bit_depth, t.channels, t.bitrate, t.album_id, t.cover_id, t.status,
       t.added_at,
       COALESCE((SELECT value FROM overrides o WHERE o.track_id = t.id AND o.field = 'title'), t.title) AS title,
       COALESCE((SELECT value FROM overrides o WHERE o.track_id = t.id AND o.field = 'artist'), t.artist) AS artist,
       COALESCE((SELECT value FROM overrides o WHERE o.track_id = t.id AND o.field = 'album'), t.album) AS album,
       COALESCE((SELECT value FROM overrides o WHERE o.track_id = t.id AND o.field = 'album_artist'), t.album_artist) AS album_artist,
       COALESCE((SELECT CAST(value AS INTEGER) FROM overrides o WHERE o.track_id = t.id AND o.field = 'track_number'), t.track_number) AS track_number,
       COALESCE((SELECT CAST(value AS INTEGER) FROM overrides o WHERE o.track_id = t.id AND o.field = 'disc_number'), t.disc_number) AS disc_number,
       COALESCE((SELECT CAST(value AS INTEGER) FROM overrides o WHERE o.track_id = t.id AND o.field = 'year'), t.year) AS year,
       COALESCE((SELECT value FROM overrides o WHERE o.track_id = t.id AND o.field = 'genre'), t.genre) AS genre,
       COALESCE((SELECT value FROM overrides o WHERE o.track_id = t.id AND o.field = 'composer'), t.composer) AS composer
FROM tracks t;

-- Full-text search over the displayed values (SPEC §5.3), overrides
-- included: the triggers index from track_view, and setting an override
-- refreshes the track's entry.
CREATE VIRTUAL TABLE tracks_fts USING fts5(
    title, artist, album, album_artist, genre,
    tokenize = 'unicode61 remove_diacritics 2'
);

CREATE TRIGGER tracks_fts_insert AFTER INSERT ON tracks BEGIN
    INSERT INTO tracks_fts(rowid, title, artist, album, album_artist, genre)
    SELECT id, title, artist, album, album_artist, genre FROM track_view WHERE id = new.id;
END;

CREATE TRIGGER tracks_fts_update AFTER UPDATE OF title, artist, album, album_artist, genre ON tracks BEGIN
    DELETE FROM tracks_fts WHERE rowid = old.id;
    INSERT INTO tracks_fts(rowid, title, artist, album, album_artist, genre)
    SELECT id, title, artist, album, album_artist, genre FROM track_view WHERE id = new.id;
END;

CREATE TRIGGER tracks_fts_delete AFTER DELETE ON tracks BEGIN
    DELETE FROM tracks_fts WHERE rowid = old.id;
END;
"#;

const V2: &str = r#"
-- Playlists, manual and smart (SPEC §6.2, §6.3).
CREATE TABLE playlists (
    id          INTEGER PRIMARY KEY,
    name        TEXT NOT NULL,
    kind        TEXT NOT NULL CHECK (kind IN ('manual', 'smart')),
    rules       TEXT,                         -- JSON, smart playlists only
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

-- A manual playlist has an order of its own and may hold the same track
-- more than once, so the position is part of the key rather than the track
-- (SPEC §6.2).
CREATE TABLE playlist_items (
    playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    position    INTEGER NOT NULL,
    track_id    INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    PRIMARY KEY (playlist_id, position)
);
CREATE INDEX playlist_items_track ON playlist_items(track_id);
"#;
