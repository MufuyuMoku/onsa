//! Measures a first scan against an unchanged rescan of a real folder.
//!
//! ```text
//! cargo run -p onsa-library --release --example scan -- <music folder> [work dir]
//! ```
//!
//! The database and cover cache go to a fresh work directory (by default
//! `onsa-scan-example` in the system temp folder), so the first scan really
//! is a first scan. The music folder is only read.

use std::path::PathBuf;

use onsa_library::Library;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let Some(music) = args.next().map(PathBuf::from) else {
        eprintln!("usage: scan <music folder> [work dir]");
        std::process::exit(2);
    };
    let work = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("onsa-scan-example"));
    if work.exists() {
        std::fs::remove_dir_all(&work)?;
    }

    let mut library = Library::open(&work.join("library.db"), &work.join("cache"))?;
    library.add_folder(&music)?;

    let first = library.scan(|progress| {
        eprint!("\r  {} files, {} read", progress.seen, progress.read);
    })?;
    eprintln!();
    println!(
        "first scan:     {:>9.3} s  {} files, {} read, {} failed",
        first.elapsed.as_secs_f64(),
        first.seen,
        first.read,
        first.failed
    );

    let again = library.scan(|_| {})?;
    println!(
        "rescan:         {:>9.3} s  {} files, {} unchanged, {} read",
        again.elapsed.as_secs_f64(),
        again.seen,
        again.unchanged,
        again.read
    );
    let ratio = first.elapsed.as_secs_f64() / again.elapsed.as_secs_f64().max(1e-6);
    println!("rescan is {ratio:.1}x faster");
    println!(
        "tracks {}, albums {}, work dir {}",
        library.track_count()?,
        library.albums_page(0, usize::MAX >> 1)?.len(),
        work.display()
    );
    Ok(())
}
