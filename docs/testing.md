# Testing

```sh
cargo test --workspace
```

The synthetic tests (archives written by the tests themselves, hand-built resources and effects) need nothing and always run. The real-data tests read your own GTA V install and key material, and compare against CodeWalker's readings of the same install; each one skips with a line naming the missing key when its input is not configured, so the command above is complete without any game files.

To run them, write a TOML file and set `GAME_DATA` to its path:

```toml
[installs]
gta = 'D:/games/Grand Theft Auto V'    # the install folder (Enhanced or Legacy)

[keys]
gta = 'D:/keys/gta'                     # the key folder, in the layout of `Keys::from_dir`

[oracles]
root = 'D:/oracles'                     # <root>/codewalker/gta/ holds CodeWalker's dumps (optional)

[run]
require = 'gate'                        # inputs marked `gate` fail instead of skipping when missing
```

The layout of the file is that of [primase-testdata](https://github.com/Kalmarv/primase-testdata) (see its `game-data.example.toml`). [tests/inputs/README.md](../tests/inputs/README.md) lists every key, oracle file and debug key (`GAME_DATA_DEBUG`) the tests read, with what each is and where it comes from. The oracle files are produced by running [CodeWalker](https://github.com/dexyfex/CodeWalker) over the same install; they are optional, and the tests that need them skip without them.

Concurrent reads through an `Install` reuse independently checked-out archive handles from its pool; the library
opens another when none is spare. `examples/read_bench.rs` times install reads on an explicit thread count and
hashes every result, for checking a change to the archive read path.

From the repository root, `cargo test -p rage-engine-formats --doc` checks the crate documentation, and `cargo build -p rage-engine-formats --examples` builds the example programs. `tests/readme_example.rs` fails if the README example differs from `examples/pack_list.rs`.
