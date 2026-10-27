# Contributing

## Build and test

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

The toolchain is pinned in `rust-toolchain.toml`. The synthetic tests and the doc test need no game files. The real-data tests read your own GTA V install and key material through the `GAME_DATA` file described in the [README](README.md#tests) and skip, printing the key they are missing, when it is not set. Every input they read is declared in `tests/inputs/mod.rs`; after changing a declaration, the test `real_data_inputs_are_declared` tells you what `tests/inputs/README.md` must say.

The example in the README is the same code as `examples/pack_list.rs` and the crate documentation's example; a test (`tests/readme_example.rs`) fails if they differ.

## Reporting a file it cannot read

Open an issue with the "unreadable file" template. It asks for:

- the version of this crate (or the commit) and your platform and game edition (Enhanced or Legacy, and the game's build number if you know it);
- the exact message: the `Error` with its format, offset and detail, or the test's panic message;
- the file's name, its path inside the archive, its size, and what you were reading it with (`Install::open`, `resource::inflate_file`, a module's `parse`).

Do not attach game files, archives, key material or anything extracted from the game: they are not ours to redistribute, and the report does not need them. If a hex dump of a few dozen bytes around the reported offset helps, include only that.

## Input and output

The parsers take bytes. Only the modules that open the install's archives and the key files (`archive::rpf`, `archive::archive_set`, `archive::keys`, `archive::listing`) use the file system, and each says so with an `allow` and a reason; `clippy.toml` forbids file, environment and process access everywhere else. Keep it that way: a new reader takes a `&[u8]` or a decoded `Resource`.

## Changing the code

Keep readers strict (read to the last byte, refuse what the layout does not account for) and refusals specific (the format, the offset where there is one, what was expected). Use checked reads rather than indexing, document public items, and add a test next to the behaviour you change: a synthetic one where a hand-built file can show it, a real-data one where only the game's files can.
