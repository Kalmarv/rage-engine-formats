//! A timed read of a GTA V install's files, with a digest of everything it read: the benchmark for the archive read
//! path (opening the archives, decrypting and inflating binaries and resources). Run before and after a change to it:
//! the times compare, the digest must not change.
//!
//! ```text
//! cargo run --release --example read_bench -- <install folder> <key folder> \
//!     [--sample K] [--threads N] [--ext ydr,ytd,..] [--max-bytes N]
//! ```
//!
//! Every `K`th file (listing order, default 7) with one of the extensions (default `ydr,ytd,ydd,yft,ybn,ymap`) and at most
//! `--max-bytes` bytes (default 64 MiB) is read on `N` threads (default 1), each thread taking files one at a time; the
//! digest folds in each file's path and bytes in listing order. The numbers are for one process on a warm file cache:
//! state which when comparing.
#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::unwrap_used,
    reason = "a benchmark program"
)]

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use rage_engine_formats::archive::archive_set::Install;
use rage_engine_formats::archive::keys::Keys;
use rage_engine_formats::archive::listing::Storage;
use sha2::{Digest, Sha256};

/// What reading one file gave: its place in the list, the digest of its bytes and their count.
type Read = (usize, [u8; 32], u64);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let (Some(install_dir), Some(key_dir)) = (args.next(), args.next()) else {
        return Err("usage: read_bench <install folder> <key folder> [--sample K] [--threads N] [--ext ydr,ytd,..] [--max-bytes N]".into());
    };
    let (mut sample, mut threads, mut max_bytes) = (7usize, 1usize, 64u64 << 20);
    let mut extensions: Vec<String> = ["ydr", "ytd", "ydd", "yft", "ybn", "ymap"]
        .map(str::to_owned)
        .to_vec();
    while let Some(a) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{a} needs a value"));
        match a.as_str() {
            "--sample" => sample = value()?.parse::<usize>()?.max(1),
            "--threads" => threads = value()?.parse::<usize>()?.max(1),
            "--max-bytes" => max_bytes = value()?.parse()?,
            "--ext" => extensions = value()?.split(',').map(str::to_owned).collect(),
            other => return Err(format!("unknown option {other}").into()),
        }
    }
    let keys = Keys::from_dir(Path::new(&key_dir), None)?;
    let t = Instant::now();
    let install = Install::open(Path::new(&install_dir), &keys)?;
    let open = t.elapsed();
    let files: Vec<_> = install
        .listing()
        .filter(|f| extensions.iter().any(|e| e == f.extension) && f.size() <= max_bytes)
        .step_by(sample)
        .collect();
    let next = AtomicUsize::new(0);
    let t = Instant::now();
    let results: Vec<(Vec<Read>, std::time::Duration)> = std::thread::scope(|s| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                s.spawn(|| {
                    let mut out = Vec::new();
                    let busy = Instant::now();
                    loop {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        let Some(file) = files.get(i) else { break };
                        let (digest, len) = match file.storage {
                            Storage::Binary { .. } => match install.binary(file.file, &keys) {
                                Ok(bytes) => (Sha256::digest(&bytes).into(), bytes.len() as u64),
                                Err(e) => (Sha256::digest(e.to_string()).into(), 0),
                            },
                            Storage::Resource { .. } => match install.resource(file.file, &keys) {
                                Ok(resource) => {
                                    (Sha256::digest(&resource.data).into(), resource.data.len() as u64)
                                }
                                Err(e) => (Sha256::digest(e.to_string()).into(), 0),
                            },
                        };
                        out.push((i, digest, len));
                    }
                    (out, busy.elapsed())
                })
            })
            .collect();
        workers.into_iter().map(|w| w.join().unwrap()).collect()
    });
    let read = t.elapsed();
    let mut all: Vec<Read> = results.iter().flat_map(|(r, _)| r.iter().copied()).collect();
    all.sort_by_key(|r| r.0);
    let mut hasher = Sha256::new();
    let mut bytes = 0u64;
    let mut refused = 0usize;
    for (i, digest, len) in &all {
        hasher.update(files[*i].path.as_bytes());
        hasher.update(digest);
        bytes += len;
        refused += usize::from(*len == 0);
    }
    println!(
        "{} archives, {} files read of the {} listed with the extensions, 1 in {sample}, {threads} thread(s)",
        install.archives.len(),
        all.len(),
        install
            .listing()
            .filter(|f| extensions.iter().any(|e| e == f.extension))
            .count()
    );
    println!("open (archive tables)       {:8.3} s", open.as_secs_f64());
    println!(
        "read + decrypt + inflate    {:8.3} s   {:8.1} MiB out   {:8.1} MiB/s",
        read.as_secs_f64(),
        bytes as f64 / 1_048_576.0,
        bytes as f64 / 1_048_576.0 / read.as_secs_f64().max(1e-9)
    );
    println!("empty or refused: {refused}");
    println!(
        "digest {}",
        hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    Ok(())
}
