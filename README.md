# rage-engine-formats

A Rust library that reads the file formats of Grand Theft Auto V's RAGE engine. It is used by [primase](https://github.com/Kalmarv/primase), a game-asset conversion library.

## Features

- Reads RPF7 archives, including nested archives and the NG and AES ciphers
- Opens a whole install and lists every archive and file with its stored and unpacked sizes
- Applies the DLC mount data to find the live copy of each file, for the story-mode or GTA Online map
- Inflates RSC7 resources into their system and graphics pages
- Reads drawables, fragments and drawable dictionaries (`.ydr`, `.yft`, `.ydd`) with their models, geometry, shaders, skeletons and lights
- Reads texture dictionaries (`.ytd`) and the Enhanced edition's buffer formats
- Reads collision bounds (`.ybn`): primitives, polygon meshes with materials, and composites
- Reads clip dictionaries (`.ycd`) and decodes their animation channels to per-frame values
- Decodes META documents and reads map data (`.ymap`), map types (`.ytyp`) and ped variation info from them
- Reads Legacy compiled effects (`.fxc`) and the Enhanced `SGD2` shader database with its techniques, render states and binding tables
- Parses the game's XML data files, their binary RBF form and the texture dictionary parent list (`gtxd`)
- Takes archive key material from the caller and checks it against caller-supplied SHA-1 digests; the crate contains no keys
- Returns an `Error` naming the format, byte offset and expected value for malformed input instead of panicking
- Limits how much memory sizes and counts read from a file can make it allocate
- Uses no unsafe code, and only the archive and key modules touch the file system

## Getting Started

### Requirements

- Rust 1.99 or later (edition 2024)
- To open archives: a GTA V PC install and its archive key material, which you supply yourself

The resource readers use the Enhanced edition's layouts (drawables version 159, fragments 171, texture dictionaries 5, clip dictionaries 46). From the Legacy edition, the crate reads `.fxc` effect files and can find key material in the Legacy executable (`Keys::from_legacy_executable`).

### Installation

```sh
cargo add rage-engine-formats
```

### Usage

Parsing needs no game files. This program reads a DLC pack list and shows an error for malformed XML (it is `examples/pack_list.rs`):

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

```console
$ cargo run --example pack_list
["dlcpacks:/mpbeach/"]
XML at 0x6: example.xml: </a> closes <b>, expected </b>
```

To read an install, load the key material and open the folder. `Keys::from_dir` reads a folder holding `gtav_ng_key.dat`, `gtav_ng_decrypt_tables.dat`, `gtav_hash_lut.dat` and, optionally, `gtav_aes_key.dat`; `Keys::none()` opens only archives without a cipher.

```rust
use std::path::Path;

use rage_engine_formats::archive::archive_set::Install;
use rage_engine_formats::archive::keys::Keys;
use rage_engine_formats::drawable::Drawable;

let keys = Keys::from_dir(Path::new("keys/gta"), None)?;
let install = Install::open(Path::new("Grand Theft Auto V"), &keys)?;
for file in install.listing().filter(|f| f.extension == "ydr").take(5) {
    let drawable = Drawable::read_resource(&install.resource(file.file, &keys)?)?;
    println!("{}: {} shaders", file.path, drawable.shaders.len());
}
```

Other examples:

- `cargo run --example list_install -- <install folder> [<key folder>]` prints the archive count and the first files of an install.
- `cargo run --release --example read_bench -- <install folder> <key folder> [--sample K] [--threads N] [--ext ydr,ytd,..] [--max-bytes N]` times reading an install's files and prints a digest of everything read.

Run the tests with `cargo test --workspace`. Tests that read a real install skip unless `GAME_DATA` points at a configuration file; see [docs/testing.md](docs/testing.md). What the readers guarantee and leave out is in [docs/guarantees.md](docs/guarantees.md), and contributing guidelines are in [CONTRIBUTING.md](CONTRIBUTING.md).

Licensed under GPL-3.0-or-later; see [LICENSE](LICENSE).
