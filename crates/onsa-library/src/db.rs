//! Opening the library database and bringing its schema up to date.

use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::functions::FunctionFlags;
use rusqlite::{Connection, OptionalExtension};

use crate::error::{Error, Result};
use crate::schema::MIGRATIONS;

/// The music library: its database and its cover cache.
pub struct Library {
    pub(crate) conn: Connection,
    pub(crate) covers_dir: PathBuf,
}

impl Library {
    /// Opens (or creates) the library database at `db_path`, with covers
    /// cached under `cache_dir`, and applies any pending migrations.
    pub fn open(db_path: &Path, cache_dir: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
        }
        let mut conn = Connection::open(db_path)?;
        // Readers (the interface) and the scanner work side by side.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        Self::with_connection(&mut conn, cache_dir)?;
        Ok(Self {
            conn,
            covers_dir: cache_dir.join("covers"),
        })
    }

    /// An in-memory library, for tests.
    pub fn open_in_memory(cache_dir: &Path) -> Result<Self> {
        let mut conn = Connection::open_in_memory()?;
        Self::with_connection(&mut conn, cache_dir)?;
        Ok(Self {
            conn,
            covers_dir: cache_dir.join("covers"),
        })
    }

    fn with_connection(conn: &mut Connection, cache_dir: &Path) -> Result<()> {
        conn.pragma_update(None, "foreign_keys", "ON")?;
        register_functions(conn)?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.busy_timeout(Duration::from_secs(5))?;
        let covers = cache_dir.join("covers");
        std::fs::create_dir_all(&covers).map_err(|source| Error::io(&covers, source))?;
        migrate(conn)?;
        Ok(())
    }

    /// Schema version of the open database.
    pub fn schema_version(&self) -> Result<u32> {
        schema_version(&self.conn)
    }

    /// Folder the cover thumbnails live in.
    pub fn covers_dir(&self) -> &Path {
        &self.covers_dir
    }

    /// Reads a setting (a JSON value).
    pub fn setting(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .optional()?)
    }

    /// Stores a setting (a JSON value).
    pub fn set_setting(&self, key: &str, json: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [key, json],
        )?;
        Ok(())
    }
}

/// Adds `onsa_parent(path)`, the folder a file sits in, so the folder view
/// can group and filter tracks in SQL (SPEC §5.3).
fn register_functions(conn: &Connection) -> Result<()> {
    conn.create_scalar_function(
        "onsa_parent",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |context| {
            let path = context.get_raw(0).as_str()?;
            Ok(parent_of(path).to_owned())
        },
    )?;
    Ok(())
}

/// The folder part of a path, without its separator. Both separators count,
/// so a database written on one system still groups on the other.
pub(crate) fn parent_of(path: &str) -> &str {
    match path.rfind(['/', '\\']) {
        Some(0) => &path[..1],
        Some(cut) => &path[..cut],
        None => "",
    }
}

fn schema_version(conn: &Connection) -> Result<u32> {
    Ok(conn.pragma_query_value(None, "user_version", |row| row.get(0))?)
}

/// Applies the migrations the database has not seen yet, each in its own
/// transaction. Returns the resulting version.
pub(crate) fn migrate(conn: &mut Connection) -> Result<u32> {
    let known = MIGRATIONS.len() as u32;
    let found = schema_version(conn)?;
    if found > known {
        return Err(Error::UnsupportedSchema { found, known });
    }
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(found as usize) {
        let version = index as u32 + 1;
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", version)?;
        tx.commit()?;
        tracing::info!(version, "library schema migrated");
    }
    Ok(known)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("onsa ライブラリ db {} {name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_new_database_gets_every_migration() {
        let dir = temp_dir("new");
        let library = Library::open(&dir.join("library.db"), &dir).unwrap();
        assert_eq!(library.schema_version().unwrap(), MIGRATIONS.len() as u32);
        assert!(library.covers_dir().is_dir());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reopening_applies_nothing_twice() {
        let dir = temp_dir("reopen");
        let db = dir.join("library.db");
        drop(Library::open(&db, &dir).unwrap());
        let library = Library::open(&db, &dir).unwrap();
        assert_eq!(library.schema_version().unwrap(), MIGRATIONS.len() as u32);
        drop(library);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_newer_schema_is_refused_rather_than_damaged() {
        let dir = temp_dir("newer");
        let db = dir.join("library.db");
        drop(Library::open(&db, &dir).unwrap());
        let conn = Connection::open(&db).unwrap();
        conn.pragma_update(None, "user_version", 99).unwrap();
        drop(conn);
        match Library::open(&db, &dir) {
            Err(Error::UnsupportedSchema { found: 99, .. }) => {}
            other => panic!("expected UnsupportedSchema, got {:?}", other.err()),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn settings_round_trip() {
        let dir = temp_dir("settings");
        let library = Library::open_in_memory(&dir).unwrap();
        assert_eq!(library.setting("theme").unwrap(), None);
        library.set_setting("theme", "\"kaca-asap\"").unwrap();
        library.set_setting("theme", "\"deck-malam\"").unwrap();
        assert_eq!(
            library.setting("theme").unwrap().as_deref(),
            Some("\"deck-malam\"")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
