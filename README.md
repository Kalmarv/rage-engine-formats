# rage-engine-formats

A Rust library that reads the file formats of Rockstar's RAGE engine as Grand Theft Auto V stores them: bytes (or, for archives, paths) in, plain records out.

It was written and checked against GTA V Enhanced (resource layouts: drawables version 159, fragments 171, texture dictionaries 5, clip dictionaries 46); the Legacy edition is read where a module says so. Against an install, the real-data tests compare the readings with [CodeWalker](https://github.com/dexyfex/CodeWalker)'s. By module, it reads:

| module | reads |
|---|---|
| `archive`, `mount` | RPF7 archives (NG and AES ciphers, nested archives), an install's listing and file tables, and the DLC mount data that decides which archives are live |
| `resource` | RSC7 resources: the inflated system and graphics pages and the pointers into them |
| `drawable`, `texture`, `sga` | drawables and fragments (models, geometry, shaders and their parameters, skeletons, lights), texture dictionaries and their buffer formats |
| `bounds` | collision bounds: primitives, geometry bounds with polygons and materials, composites |
| `ycd`, `ycd_channels` | clip dictionaries and the decoded channel tracks of their animations |
| `meta`, `map`, `ped` | META documents, and the map data, map types and ped variation info read from them |
| `fxc`, `sgd2`, `sgd2_effect`, `sgd2_table` | Legacy's compiled effects (`.fxc`) and Enhanced's `SGD2` shader database with its techniques and binding tables |
| `xml`, `rbf`, `gtxd` | the game's XML data files, their binary form (RBF) and the texture-dictionary parent list |

The [crate documentation](https://docs.rs/rage-engine-formats) groups the modules and gives the layout each reader holds a file to. The drawable, texture, bounds, clip and shader readers use the Enhanced layouts; `fxc` reads Legacy effect files and `Keys::from_legacy_executable` finds key material in a Legacy executable.

## Example

Parsing needs no game files. This reads a DLC pack list and shows a refusal (it is `examples/pack_list.rs`, run with `cargo run --example pack_list`, and also the crate documentation's doc test):

```rust
use rage_engine_formats::{Error, mount, xml};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A DLC pack list (`dlclist.xml`): the packs the game looks for, in list order.
    let dlclist =
        br"<SMandatoryPacksData><Paths><Item>dlcpacks:\mpbeach\</Item></Paths></SMandatoryPacksData>";
    println!("{:?}", mount::pack_list(dlclist)?);

    // Malformed input is an error that says what was wrong, never a panic.
    let Err(refused) = xml::parse(b"<a><b></a>", "example.xml") else {
        unreachable!("the closing tag does not match")
    };
    assert!(matches!(refused, Error::Invalid { format: "XML", .. }));
    println!("{refused}");
    Ok(())
}
```

To open an install, pass the folder and your key material to `Install::open` (see `examples/list_install.rs`: `cargo run --example list_install -- <install folder> <key folder>`).

## What it guarantees

- **Checked reads.** The format readers take a byte slice or a decoded `Resource` and read through bounds-checked accessors; the whole library denies unchecked indexing, slicing, arithmetic and narrowing casts by lint (the tests, which build their fixtures, are outside it). A file that does not hold what its format says is an `Error`, not a panic, and a pointer, count or size read from a file is checked against the bytes there are before it is followed. How much a file can make a reader allocate is limited: a resource inflates to at most its compressed size times a fixed ratio plus a fixed allowance, and reads that copy out of a resource are charged against it and refused beyond four times its size.
- **Refusals that say what was wrong.** `Error::Invalid` carries the format, the byte offset and a description of what was expected. The XML-based files (`xml`, `mount`) report the offset in the document of the markup that is wrong.
- **Strict readers.** The whole-file formats (`fxc`, `sgd2`, `sgd2_table`, `rbf`) are read to their last byte, bytes after the last record are an error, and the XML-based readers refuse an element they do not know instead of skipping it, so a layout that differs from what the reader was checked against fails loudly.
- **No unsafe code, no hidden I/O.** `unsafe` is denied; only the modules that open archives and key files touch the file system, and none read the environment or start processes.
- **No key material.** The archive ciphers need keys that this crate does not contain and cannot derive: you supply them as bytes or as a folder (`archive::keys::Keys::from_parts`, `from_dir`), and may supply the SHA-1 digests they must match (`archive::keys::KeyDigests`). The crate holds no digests either.

## What it does not do

- It does not write or modify anything: there are no writers for any format.
- It does not ship, find or download keys, and it does not include or read game files other than the ones you point it at.
- It does not decode pixels or render: a texture is its description and its stored bytes; a drawable is its vertex and index buffers and shader parameters. Parts of the formats that its documentation lists as not read (the collision BVH tree, clip tags and properties, map occluders and other extensions) are left as stored or as META trees.
- It does not interpret what a value means beyond the layout (flags, hashes and enums stay raw unless a type says otherwise).
- It reads GTA V only; other RAGE games are untested.

## Tests

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

The layout of the file is that of [primase-testdata](https://github.com/Kalmarv/primase-testdata) (see its `game-data.example.toml`). [tests/inputs/README.md](tests/inputs/README.md) lists every key, oracle file and debug key (`GAME_DATA_DEBUG`) the tests read, with what each is and where it comes from. The oracle files are produced by running [CodeWalker](https://github.com/dexyfex/CodeWalker) over the same install; they are optional, and the tests that need them skip without them.

Concurrent reads through an `Install` reuse independently checked-out archive handles from its pool; the library
opens another when none is spare. `examples/read_bench.rs` times install reads on an explicit thread count and
hashes every result, for checking a change to the archive read path.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md), including how to report a file this library cannot read.

## Check the examples

From the repository root, `cargo test -p rage-engine-formats --doc` checks the crate documentation, and
`cargo build -p rage-engine-formats --examples` builds its example programs. Examples that read paths need the caller's files; the repository ships no game assets.

## License

GPL-3.0-or-later; see [LICENSE](LICENSE).

Used by [primase](https://github.com/Kalmarv/primase), a game-asset conversion library.
