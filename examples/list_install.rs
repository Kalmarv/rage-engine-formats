//! Lists the archives of a GTA V install and a few of the files in them.
//!
//! `cargo run --example list_install -- <install folder> [<key folder>]`
//!
//! Without a key folder only archives with no cipher can be opened; an install needs its key material (see
//! `rage_engine_formats::archive::keys`).
use std::path::Path;

use rage_engine_formats::archive::archive_set::Install;
use rage_engine_formats::archive::keys::Keys;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let install_dir = args
        .next()
        .ok_or("usage: list_install <install folder> [<key folder>]")?;
    let keys = match args.next() {
        Some(dir) => Keys::from_dir(Path::new(&dir), None)?,
        None => Keys::none(),
    };
    let install = Install::open(Path::new(&install_dir), &keys)?;
    println!("{} archives", install.archives.len());
    for file in install.listing().take(10) {
        println!("{} ({} bytes)", file.path, file.size());
    }
    Ok(())
}
