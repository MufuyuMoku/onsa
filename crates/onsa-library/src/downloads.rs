//! The download queue, kept so it outlives Onsa being closed (SPEC §7.3).
//!
//! The table holds exactly what is left to do: a row is added when a
//! download is asked for and removed when it ends, however it ends. Onsa
//! closed part way leaves the rows of what had not finished, and those are
//! what is offered again when it opens.
//!
//! Each row carries the format and the folder it was asked for with, so a
//! download picked up again lands where, and as what, it would have.

use rusqlite::params;

use crate::{Library, Result};

/// One download waiting to be done.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedDownload {
    /// Its row, which is also how the queue on screen follows it.
    pub id: i64,
    /// The URL of this one item.
    pub url: String,
    /// What it is called, as far as anyone knows yet.
    pub title: String,
    /// The format asked for, by its name.
    pub format: String,
    /// Where the finished file goes.
    pub folder: String,
}

impl Library {
    /// Remembers a download that has been asked for. Answers with its row.
    pub fn queue_download(
        &mut self,
        url: &str,
        title: &str,
        format: &str,
        folder: &str,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO download_queue (url, title, format, folder, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![url, title, format, folder, now_ms()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Writes down the name a download turned out to have, so the queue
    /// offered after a restart says what it is rather than what was guessed.
    pub fn retitle_queued_download(&mut self, id: i64, title: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE download_queue SET title = ?2 WHERE id = ?1",
            params![id, title],
        )?;
        Ok(())
    }

    /// Forgets a download, because it is over: done, failed or stopped.
    pub fn forget_queued_download(&mut self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM download_queue WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Everything still to do, oldest first.
    pub fn queued_downloads(&self) -> Result<Vec<QueuedDownload>> {
        let mut statement = self.conn.prepare(
            "SELECT id, url, title, format, folder FROM download_queue ORDER BY added_at, id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(QueuedDownload {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                format: row.get(3)?,
                folder: row.get(4)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }
}

/// Now, in milliseconds since the Unix epoch.
fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_millis() as i64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_was_left_to_do_is_still_there_when_the_library_opens_again() {
        let dir =
            std::env::temp_dir().join(format!("onsa downloads queue {} 音叉", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let db = dir.join("library.db");
        let folder = dir.join("Unduhan").display().to_string();

        let (first, second) = {
            let mut library = Library::open(&db, &dir.join("covers")).unwrap();
            let first = library
                .queue_download("https://example.com/1", "satu", "mp3", &folder)
                .unwrap();
            let second = library
                .queue_download("https://example.com/2", "dua", "original", &folder)
                .unwrap();
            let third = library
                .queue_download("https://example.com/3", "tiga", "flac", &folder)
                .unwrap();
            library
                .retitle_queued_download(second, "Dua, judul aslinya")
                .unwrap();
            // The third finished before Onsa was closed.
            library.forget_queued_download(third).unwrap();
            (first, second)
        };

        let library = Library::open(&db, &dir.join("covers")).unwrap();
        let left = library.queued_downloads().unwrap();
        assert_eq!(
            left,
            vec![
                QueuedDownload {
                    id: first,
                    url: "https://example.com/1".into(),
                    title: "satu".into(),
                    format: "mp3".into(),
                    folder: folder.clone(),
                },
                QueuedDownload {
                    id: second,
                    url: "https://example.com/2".into(),
                    title: "Dua, judul aslinya".into(),
                    format: "original".into(),
                    folder,
                },
            ]
        );
        drop(library);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_id_is_never_given_out_twice_even_once_the_queue_is_empty() {
        let mut library = Library::open_in_memory(&std::env::temp_dir()).unwrap();
        let first = library
            .queue_download("https://example.com/1", "satu", "mp3", "/x")
            .unwrap();
        library.forget_queued_download(first).unwrap();
        assert!(library.queued_downloads().unwrap().is_empty());
        let second = library
            .queue_download("https://example.com/2", "dua", "mp3", "/x")
            .unwrap();
        assert_ne!(first, second, "the screen would take them for one download");
    }
}
