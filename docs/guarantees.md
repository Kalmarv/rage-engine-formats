# Guarantees and limits

What the readers of rage-engine-formats guarantee, and what they do not do.

## Guarantees
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
